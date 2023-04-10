# High-level

The main external interface here is ActionCoordinator trait,
which consumes Actions and determines what Actions will exist in a given tick.
ConnectionToHost and HostingSession (which has many ClientConnections) take care of the relevant side
of this flow.

## ConnectionToHost

- Exists on each client
- Implements ActionCoordinator
- Takes actions that will be sent to the host at the end of the current tick
- Sends actions to the host
- Receives actions from the host for "finalized" ticks, and buffers them until that time

## HostingSession

- Exists on the host
- Implements ActionCoordinator
- Takes Actions from the local "client" and actions from the owned ConnectionToClients
  and passes them to the tick coordinator
- At the end of a tick, gets any finalized ticks from the tick coordinator and sends
  them to all clients

## ConnectionToClient

- An instance exists on the host for each connected client
- Receives Actions from a particular client and holds them for the owning HostingSession