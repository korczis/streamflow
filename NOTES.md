## Commands Alternatives
- streamflow wc -l ./data/sample.csv

## Real World

- Read CSV
  - Generate Meta-information (Manifest) [OPTIONAL]
- Transpose
- Per column streaming
- Per stream stat
- Aggregate

## Iterators in Functions

`iterator -> f(context, data) -> iterator`

let reducer_stats = data::reducer::Stats();
let source_csv = data::producer::file::Csv::new({});

## Reverse notation

```
reducer_stats
.from(source_csv)
.from(network_reader)
.end();
```

## Forward notation

```
network_reader
.to(source_csv)
.to(reducer_stats)

let flow_manager = FlowManager::default();
flow_manager.run([
 ReadCsv,
 Transpose,
 Serialize,
]);
```

## TODO: Create transpose block
- Create channels
- Create create thread
- Run loop

## TODO: Create CSV reading block
- Run loop
  - Send each row to first block
- Wait for chain to finish
- Collect & Interpret result