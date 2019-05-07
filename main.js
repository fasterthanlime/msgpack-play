const msgpack = require("msgpack-lite");
const { readFileSync } = require("fs");

let path = "./buf.bin";
console.log(`Reading from ${path}`);
let buf = readFileSync(path);
let msg = msgpack.decode(buf);
console.dir(msg);
