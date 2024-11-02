I want to move onto a bit more of a fun project, tcp packet handling

it won't be the full tcp, just a wrapper on udp, so not actually a transport protocol but hey

## Plan(in scope)
1. basic client/server system
2. sliding window
3. ack/retry
4. checksum
5. send the bee movie script

### Maybes
1. reset connection idk what it means tbh
2. sync seq nums, again not sure
3. connect to another computer on the same network/virtualized

## Possible ideas(out of scope, for now)
1. urgent pointers
2. interacting with the IP layer, and actually creating a tcp protocol that all computers can work with
3. maybe a tui? (later down the line I want to make my own http on this protocol, and then maybe a postman tui clone)
    but that being said, a tui dashboard of acks rolling through, throughput of packets, etc, could be siiiiick