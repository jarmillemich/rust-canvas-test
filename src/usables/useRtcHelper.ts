// Currently built to work with the demo server from https://github.com/jarmillemich/spin-rtc-signaling

export function useRtcHelper() {
  return {
    startHostingSession,
    joinSession,
  }
}

const stunUrl = 'stun:stun3.l.google.com:19302'
const signallingUrl = 'https://jarmillemich-rust-signaling.fermyon.app'

function log(msg: string | ArrayBuffer) {
  console.log(`[RTC] ${msg}`)
  if (msg instanceof ArrayBuffer) {
    console.log(` => ${new TextDecoder().decode(msg)}`)
  }
}

/**
 * Delay the specified number of milliseconds and resolve
 */
function delay(ms: number) {
  return new Promise((resolve, _reject) => setTimeout(resolve, ms))
}

interface ChannelPair {
  connection: RTCPeerConnection
  channel: RTCDataChannel
}

/**
 * Attaches to an RTCPeerConnection to gather ICE candidates.
 * Resolves with the array of gathered candidates when we receive an "End of Candidates" message
 */
function gatherIceCandidates(connection: RTCPeerConnection): Promise<Array<RTCIceCandidate>> {
  let state = connection.iceGatheringState

  // We should attach right away or we could miss some candidates
  if (state !== 'new') throw new Error('It is probably a bad idea to start gathering ICE candidates after the connection is gathering')

  return new Promise((resolve, reject) => {
    // We have some goofy sentinel values to signal state changes, sometimes
    // See https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API/Connectivity#choosing_a_candidate_pair
    let candidates: RTCIceCandidate[] = []

    connection.onicecandidate = event => {
      if (event.candidate === null) {
        // This one does not need to be sent
        log('Null EoC message')
        return
      }

      log('Received local ICE candidate!')
      console.log('Got candidate', event.candidate)
      candidates.push(event.candidate)

      // Empty string is the "end of candidates" message
      // Supposedly we should send this one too
      // @ts-expect-error - TS doesn't know that this is a valid (end) candidate
      if (event.candidate === '') {
        log('EoC message')
      }
    }

    connection.onicegatheringstatechange = event => {
      console.log('ICE state change', connection.iceGatheringState)

      if (connection.iceGatheringState === 'complete') {
        resolve(candidates)
      }
    }

    connection.onicecandidateerror = (event: Event) => {
      console.error(event)
      // TODO types
      let err = event as any as RTCPeerConnectionIceErrorEventInit
      log(`Ice error (non-fatal) ${err.errorCode}: ${err.errorText}`)

    }
  })

}

class ClientConnection {
  constructor(public readonly client_name: string, private readonly onConnected: (info: ChannelPair) => void) {
    this.connection.ondatachannel = dc => {
      this.isConnected = true
      this.onDataChannel(dc.channel)
    }

    this.iceGather = gatherIceCandidates(this.connection)

  }

  private connection = new RTCPeerConnection({
    iceServers: [{ urls: stunUrl }]
  })

  public isConnected = false;
  public dc: RTCDataChannel | null = null;
  iceGather: Promise<Array<RTCIceCandidate>>

  /** Connects our connection the given offer, sets up an ICE channel, and returns an answer */
  async connectToOffer(client_offer: RTCSessionDescriptionInit) {
    await this.connection.setRemoteDescription(client_offer)
    let answer = await this.connection.createAnswer()
    this.connection.setLocalDescription(answer)
    return answer
  }

  async onIceCandidates(candidate: RTCIceCandidateInit) {
    log(`Got client ICE from ${this.client_name}`)
    await this.connection.addIceCandidate(candidate)
  }

  onDataChannel(dc: RTCDataChannel) {
    log('Got data channel!')
    dc.send('hello from the server')
    this.onConnected({ connection: this.connection, channel: dc })

    this.dc = dc;
  }
}

interface ConnectionInitMessage {
  type: 'start_join',
  client_name: string,
  client_offer: string,
  candidates: string[],
}

async function startHostingSession(onClient: (client: ChannelPair) => void) {
  log('Starting up a session')

  let res = await fetch(`${signallingUrl}/host`, {
    method: 'POST',
    body: JSON.stringify({
      public: false,
      host_name: 'Larry',
    })
  })

  log(`  Got response ${res.statusText}`);

  let session_name: string, host_secret: string;

  ({
    session_name,
    host_secret
  } = await res.json());

  await navigator.clipboard.writeText(session_name)

  log(`  session_name=${session_name}; host_secret=${host_secret}`)
  log('  (We copied it to your clipboard)')

  let clients = new Map<string, ClientConnection>()

  // Start polling for client info
  for (let i = 0; i < 10; i++) {
    await delay(1000);
    if ([...clients.values()].some(p => p.isConnected)) break

    let res = await fetch(`${signallingUrl}/host/messages?session_name=${session_name}&host_secret=${host_secret}`)
    let messages: ConnectionInitMessage[] = (await res.json()).flat().map((m: string) => JSON.parse(m));

    // Probably have to process start_join first (TODO check on this)
    messages.sort((a, b) => +(b.type === 'start_join') - +(a.type === 'start_join'))

    console.log(messages)

    for (let message of messages) {
      if (message.type === 'start_join') {
        let { client_name, client_offer } = message
        log(`Got join request from ${client_name}`)
        let client = new ClientConnection(client_name, onClient)
        clients.set(client_name, client)
        let answer = await client.connectToOffer(JSON.parse(client_offer))
        log('Sending answer')

        // Send back the answer
        await fetch(`${signallingUrl}/join/response`, {
          method: 'POST',
          body: JSON.stringify({
            session_name,
            client_name,
            host_secret,
            messages: {
              type: 'answer',
              answer
            }
          })
        })

        // Send back our candidates
        let hostCandidates = await client.iceGather
        log(`Host is sending ${hostCandidates.length} candidates`)
        fetch(`${signallingUrl}/join/response`, {
          method: 'POST',
          body: JSON.stringify({
            session_name,
            client_name,
            host_secret,
            messages: {
              type: 'ice_candidate',
              candidate: hostCandidates.map(c => JSON.stringify(c))
            }
          })
        })
      } else if (message.type === 'ice_candidate') {
        let { client_name, candidates } = message
        log(`Host got ice candidates for ${client_name}`)
        if (!clients.has(client_name)) {
          console.warn('No such client?')
          continue
        }
        for (let candidate of candidates) {
          let client = clients.get(client_name)
          if (!client) throw new Error('No such client?')
          client.onIceCandidates(JSON.parse(candidate))
        }
      } else {
        log(`Unknown message type ${message.type}`)
      }
    }

    log('Got hosting messages ' + messages.length)
  }
}

function joinSession(session_name: string, client_name: string) {
  return new Promise<ChannelPair>(async (resolve, reject) => {
    let connection = new RTCPeerConnection({
      iceServers: [{ urls: stunUrl }]
    })

    let iceGather = gatherIceCandidates(connection);

    log(`Connecting to ${session_name}`)

    log('  Creating offer')

    let connected = false

    let channel = connection.createDataChannel('main')
    channel.onopen = () => {
      log('Connected!')
      connected = true
    }

    channel.onmessage = event => {
      log(event.data)
    }


    let offer = await connection.createOffer()
    await connection.setLocalDescription(offer)

    if (!offer.sdp) throw new Error('No sdp?')

    log(offer.sdp)

    let res = await fetch(`${signallingUrl}/join`, {
      method: 'POST',
      body: JSON.stringify({
        session_name,
        client_name,
        rtc_offer: JSON.stringify(offer) // yup
      })
    })

    if (res.status !== 200) {
      log(`Failed to connect: ${res.statusText}`)
    }

    // Will get an id from the server to send candidates to
    let { client_secret } = await res.json()

    // Get our ICE canidates together first
    let ourCandidates = await iceGather
    // Send candidates to the server until we can connect
    log(`Client sending ${ourCandidates.length} candidates`)

    await fetch(`${signallingUrl}/join/candidates`, {
      method: 'POST',
      body: JSON.stringify({
        session_name,
        client_name,
        client_secret,
        candidates: ourCandidates.map(c => JSON.stringify(c))
      })
    })


    // Gotta keep these as they can happen to come in out of order
    let candidateCache = []

    // Start polling for host info
    for (let i = 0; i < 10; i++) {
      await delay(1000);
      if (connected) break;

      let res = await fetch(`${signallingUrl}/join/messages?session_name=${session_name}&client_name=${client_name}&client_secret=${client_secret}`)
      let messages = (await res.json()).flat().map((m: string) => JSON.parse(m))

      for (let message of messages) {
        if (message.type === 'answer') {
          let { answer } = message
          log(`Got host description, ${candidateCache.length} queued ICE to apply`)
          await connection.setRemoteDescription(answer)

          // Add any queued up ice candidates
          while (candidateCache.length) {
            await connection.addIceCandidate(candidateCache.shift())
          }
        } else if (message.type === 'ice_candidate') {
          let { candidate } = message

          let candidates = candidate.map((c: string) => JSON.parse(c))

          log('Got host ICE')
          for (let candidate of candidates) {
            if (connection.currentRemoteDescription) {
              log('  Adding now')
              await connection.addIceCandidate(candidate)
            } else {
              log('  Adding later')
              candidateCache.push(candidate)
            }
          }
        } else {
          log(`Unknown client message ${message.type}`)
        }
      }
    }

    if (connected) {
      log('Client side is good to go!')
      resolve({ connection, channel })
    } else {
      log('Giving up!')
      reject()
    }
  })
}