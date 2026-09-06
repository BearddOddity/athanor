import { Buffer } from 'buffer';

console.log('Testing Buffer...');

const data = Buffer.alloc(100);
console.log('Buffer.alloc works');

data.writeUInt32LE(123, 0);
console.log('writeUInt32LE works');

data.writeUInt32LE(Buffer.alloc(4), 4);
console.log('Nested Buffer.alloc works');

const num = data.readUInt32LE(0);
console.log('readUInt32LE works:', num);

console.log('All tests passed!');