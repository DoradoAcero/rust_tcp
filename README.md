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


Got Something basic working
it is stuuuupid slow though, only like 89MB/s ~ 719Mb/s
this should be orders of magnitude faster, did some testing of the std implementations of these tcdp sockets and it appears it is around 10-100x faster
not bad for a first go, not great, might go back to it
but this is fast enough to put http on.

to be able to put http on it though, it needs a full duplex, not the send only, and recieve only
I need to be able to say to the socket, wait for a string to come in
and send a string