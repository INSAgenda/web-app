#!/bin/sh

file=$(find dist -name '*.js')

to_replace="= new BigInt64Array();"
replacement="= undefined; try { cachedBigInt64Memory0 = new BigInt64Array(); } catch (e) { console.error('BigInt64Array is not supported'); }"
sed -i -e "s/$to_replace/$replacement/g" $file

to_replace="= new BigInt64Array(wasm.memory.buffer);"
sed -i -e "s/$to_replace/$replacement/g" $file
