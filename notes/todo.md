## Config 

- [ ] convert to a shared actor to include in rocket
- [ ] add session table with a loader / saver
  - [ ] own docs  and author per user
- [ ] Node cache
- [x] Secret author keys bound to documents
- [ ] Session information ( construct on request ) 
- [ ] make a [wot](https://en.m.wikipedia.org/wiki/Web_of_trust)
- [ ] keep a list of nodes and  timestamps and status 

## Application

- [ ] front page should be more helpful 
- [ ] fix the replicator
  - [ ] downloader 
  - [ ] author replication
  - [ ] share documents
- [ ] share blobs and docs over the liminal:: gossip channel
- [ ] convert web interface to be user based
- [ ] replicate documents to other nodes.
- [ ] display better node info

## Files 

- [ ] Better link for incoming docs
- [ ] Convert incoming to a __docs__ object and prime the process in the background , see [services](services).
---
# DONE 

## Notes
- [x] blank notes should error , use a Result<Redirect,Template> struct so it does not get a type error