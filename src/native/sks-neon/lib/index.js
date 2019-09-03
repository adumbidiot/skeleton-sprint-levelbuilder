var addon = require('../native');

console.log(addon.hello());
let data = ['b0', 'a1'];
console.log(addon.export1DPatch(data));

module.exports.hello = addon.hello;

module.exports.encodeBlockLBL = addon.encodeBlockLBL;
module.exports.decodeBlockLBL = addon.decodeBlockLBL;