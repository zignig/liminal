# TODO

## Notes

- [ ] Discuss key prefix  [issue](https://github.com/n0-computer/iroh-docs/issues/55) on discord
- [ ] investigate postfixing all id's with a \0x0 null char
- [ ] Display share ticket on the notes page

## Admin

- [x] Add administrator page
- [ ] __must__ be authenticated
- [x] add links to other pages

## Config 

- [ ] convert to a shared actor to include in rocket
- [ ] add session table with a loader / saver
  - [ ] own docs  and author per user
- [ ] Node cache
- [x] Secret author keys bound to documents
- [ ] Session information ( construct on request ) 
- [ ] make a [wot](https://en.m.wikipedia.org/wiki/Web_of_trust)
- [ ] keep a list of nodes and  timestamps and status 

## Replicator 

See [replicator](replicator)

- [ ] fix the replicator
  - [ ] downloader 
  - [ ] author replication
  - [ ] share documents

## Application

- [ ] front page should be more helpful 
- [x] Add search box on the front page
- [ ] share blobs and docs over the liminal:: gossip channel
- [ ] convert web interface to be user based
- [ ] replicate documents to other nodes.

## Network

- [ ] display better node info

## Files 

- [ ] Better link for incoming docs
- [ ] Convert incoming to a __docs__ object and prime the process in the background , see [services](services).
---
# TODONE 

## Notes
- [x] blank notes should error , use a Result<Redirect,Template> struct so it does not get a type error

## Config

## Application

## Files

