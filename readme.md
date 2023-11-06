experimenting with making a switch file formats driver in rust that easily supports nested containers transparently. 

this isnt really viable until i added a crypto layer implementation and verified with benchmarks that my design approach is not awful

the api i want mandates type erasure which is how i ended up with the current type erased Storage struct and the duplicating IStorage interface, its sadly a compromise because i want to support usecases where you e.g. recursively open storages and dont have to know the exact nesting and types at compile time. 
im not too happy with this design but it works for now and it makes it easy to swap out all uses of Arc with Rc in a few spots, if atomics eventually prove to become a bottleneck.
an alternative solution i thought about but havent tested yet (it takes quite a while to write all the code to test all parts of the api i want to have) is something like enum_dispatch which gives you static typing and then have one variant that is the equivalent of the type erased Storage struct that we have at the moment
