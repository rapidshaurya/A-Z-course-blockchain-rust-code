## HAD_COIN
- This example contains two chains
- chain1 run on port http://127.0.0.1:8080
- chain2 run on port http://127.0.0.1:8000
#### Run chains command
- cargo run --example chain1
- cargo run --example chain2
- run the above command inside the chain folder on different terminals

#### Chain details
These chains are similiar chain in udemy a-z blockchain course
- [course link](https://www.udemy.com/course/build-your-blockchain-az/)
- [this course is also available at youtube](https://www.youtube.com/watch?v=dn1QsirJ8gk)

#### Warning 
- Do not add nodes other then the above ports( otherwise, it will result in bad request)
- Suppose if you are running chain2, then do not added chain2 port in node(using connect node route) otherwise it will no result in replace chain route(request never stop, it will stuck in await)