## Vanity Deployer CLI

Uses max available threads 
Save Creation Code in a `.txt` file without the `0x` prefix in `/contracts` and replace this line 
```rs
let _creation_code_hex = fs::read_to_string("./contracts/DeployerContractCreationCode.txt")
    .expect("DeployerContractCreationCode.txt file not found.")
    .trim()
    .to_string();
```

save and run in the shell
```
cargo run
```
