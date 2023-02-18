Will attempt to implement a network stack similar to that described in https://www.factorio.com/blog/post/fff-149

Abbreviated path:
- Event happens on client (e.g. key press, mouse movement)
- Event converted to an action (jump, move crosshairs)
- Action encapsulated and sent to server
    - Includes client tick number as of initiating the event
    - And a sequence number so we don't do things out of order?

- Server gathers up all the actions it has and applies them
    - Don't apply something with a sequence number indicating lost actions?
    - Don't apply things originating fewer than N ticks ago
        - Calibrate based on how latent everyone is?


Probably we can use serde flexbuffer for network transerialization