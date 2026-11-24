<img width="1247" height="477" alt="image" src="https://github.com/user-attachments/assets/8d05e75a-c22c-4f80-a4c8-16813e24c37a" />

## what for? 

Wanna run an actual consensus version of GETH on your system? 

Chain being forwarded using the consensus methods:

<img width="1614" height="448" alt="image" src="https://github.com/user-attachments/assets/b794ebb2-db1c-4097-bb0b-202e04618b79" />

Before "The Merge," using GETH, it was possible to run an actual private Ethereum blockchain on your system, but apparently after the merge, unless using the "--dev" mode, which is more of like a simulation, 
it is not possible to run an actual private Ethereum node using geth.

Atm, unless you want to spin up multiple containers at the same time using kurtosis, it is not possible to run the consensus version of geth. I mean, you want to spin up 
an execution client and a consensus client on your system? Apparently with geth and the most popular consensus client lighthouse, it is not possible to do this...

For those wanting to understand how the consensus client controls the execution client, this project is going to be a goldmine, because you can see the actual flow happening behind the scenes, I mean how the forkchoice methods are called, the execution payload is built, and how a block is finalized, etc.

After building this project and finally getting the chain running, I came across a deep issue: transactions are not being executed! I mean, what...? The blocks are being built up.
and chain moving forward, but the transactions are stuck. I started debugging the whole thing, and it was due to the "EIP-1559 dynamic fees" limitations, where the miner tip goes so
low to a point where transactions will always be in a pending state or just removed since 99% blocks produced are just empty.

So this project requires a patched version of geth :/ If you wanna execute transactions, I mean you need to patch geth before using it.

Currently this is the version of geth being tested:

```bash
pv@arch ~/g/go-ethereum (master)> ./build/bin/geth --version
geth version 1.17.11-unstable-454de426-20261124
```
So a slight patch to fix the issue:

Clone the go-ethereum repo and in the legacypool.go Pending() function just uncomment this one line:

```go

		// If the miner requests tip enforcement, cap the lists now
		if filter.MinTip != nil || filter.GasLimitCap != 0 {
			for i, tx := range txs {
				fmt.Println("MinTip:", filter.MinTip.String())
				fmt.Println("BaseFee:", filter.BaseFee.String())

				if filter.MinTip != nil {
					if tx.EffectiveGasTipIntCmp(filter.MinTip, filter.BaseFee) < 0 {
						// txs = txs[:i] <------------- uncomment this
						break
					}
				}
				if filter.GasLimitCap != 0 {
					if tx.Gas() > filter.GasLimitCap {
						txs = txs[:i]
						break
					}
				}
			}
```

This particular line is the one that removes the transaction, and apparently I'm not able to find a way to send transactions without doing so. Again, this is only necessary if you want to run transactions or just want to understand how the consensus works, there is no need to patch this thing.

Aight now let's start on how to get this thing running.

1. Generate the jwt file so the consensus requests are authenticated

```go
openssl rand -hex 32 > jwt.hex
```

2. Create a genesis.json file initialized the block 0:

```json
{
  "config": {
    "chainId": 777,
    "homesteadBlock": 0,
    "eip150Block": 0,
    "eip155Block": 0,
    "eip158Block": 0,
    "byzantiumBlock": 0,
    "constantinopleBlock": 0,
    "petersburgBlock": 0,
    "istanbulBlock": 0,
    "berlinBlock": 0,
    "londonBlock": 0,
    "mergeNetsplitBlock": 0,
    "shanghaiTime": 0,
    "cancunTime": 0,
    "terminalTotalDifficulty": 0,
    "terminalTotalDifficultyPassed": true,
    "blobSchedule": {
      "cancun": {
        "target": 3,
        "max": 6,
        "baseFeeUpdateFraction": 3338477
      }
    }
  },
  "alloc": {
    "0x14e5b4Acb55c9Fe7e4f48BBc2a25Edf8577bA52c": {
      "balance": "1000000000000000000000"
    }
  },
  "difficulty": "0x1",
  "gasLimit": "0x1c9c380",
  "baseFeePerGas": "0x3B9ACA00"
}
```

3. Start geth!

```bash
build/bin/geth --datadir ./bowbow \
                                               --nodiscover \
                                               --miner.gasprice 0 \
                                               --maxpeers 0 \
                                               --http \
                                               --txpool.pricelimit 1 \
                                               --http.api eth,net,web3 \
                                               --authrpc.addr localhost \
                                               --authrpc.port 8551 \
                                               --authrpc.jwtsecret ./jwt.hex \
                                               --syncmode full 


```
Which will spin up geth and it will be waiting to receive the requests from the beacon. Compile this project using cargo build, then copy that jwt.hex file in the same
dir as the executable.

Then just run!

```bash
pv@arch ~/driver (main) [SIGINT]> ./target/debug/drivergeth


   ██████╗ ███████╗████████╗██╗  ██╗
  ██╔════╝ ██╔════╝╚══██╔══╝██║  ██║
  ██║  ███╗█████╗     ██║   ███████║
  ██║   ██║██╔══╝     ██║   ██╔══██║
  ╚██████╔╝███████╗   ██║   ██║  ██║
   ╚═════╝ ╚══════╝   ╚═╝   ╚═╝  ╚═╝

             D R I V E R

Geth Online, Starting the beacon...
```

Will will start the tiniest consensus client possible!

Going to execute a transaction now:

```bash
pv@arch ~> cast send 0x67892359f9b3ca62335dc9525d0bc0be55f93148 \
                 --value 1ether \
                 --private-key 0xc077c9d83649dea92766f73b6eeff345375780967db706f0144a02f1a9e277e6 \
                 --rpc-url http://localhost:8545

blockHash            0x688e3da2d96d41de2bf57a8f28f4403bc336e59b20c5841e5430266a711b0a71
blockNumber          320
contractAddress
cumulativeGasUsed    21000
effectiveGasPrice    8
from                 0x14e5b4Acb55c9Fe7e4f48BBc2a25Edf8577bA52c
gasUsed              21000
logs                 []
logsBloom            0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
root
status               1 (success)
transactionHash      0x12b0a5498dde6eb6da33c79b800afbd60a8f3856e933af8e373ffd5823b453e7
transactionIndex     0
type                 2
blobGasPrice
blobGasUsed
to                   0x67892359f9B3cA62335dc9525d0BC0Be55F93148
pv@arch ~>

```
