#!/bin/sh

file=$(find dist -name '*.js')

to_replace="cachedBigInt64Memory0 = new BigInt64Array();"
replacement="cachedBigInt64Memory0 = undefined; try { cachedBigInt64Memory0 = new BigInt64Array(); } catch (e) { console.error('BigInt64Array is not supported'); }"
sed -i -e "s/$to_replace/$replacement/g" $file

to_replace="cachedBigInt64Memory0 = new BigInt64Array(wasm.memory.buffer);"
replacement="cachedBigInt64Memory0 = undefined; try { cachedBigInt64Memory0 = new BigInt64Array(wasm.memory.buffer); } catch (e) { console.error('BigInt64Array is not supported'); }"
sed -i -e "s/$to_replace/$replacement/g" $file
