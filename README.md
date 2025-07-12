
---

## Security

- **No private keys or sensitive information are included in this repository.**
- The upgrade authority is controlled by the deployer’s wallet; 

---

## Verified Builds

To verify this program on Solana explorers (like Solscan):

1. Build using the [solana-verify](https://github.com/solana-developers/solana-verify) tool:
   ```sh
   cargo install solana-verify
   solana-verify build
   ```
2. Deploy using the `.so` file produced by the verified build.
3. Use the verification command:
   ```sh
   solana-verify verify-from-repo -u https://api.mainnet-beta.solana.com --program-id 47YGQvDJJzMAAq7Z6x7LegYhS5Dremk5sGRYGjkAM7c2 https://github.com/Nexgent-ai/Nexgent-token-lock.git
   ```
4. Follow explorer instructions to complete verification.

---

## Contributing

Pull requests and community reviews are welcome!  
Please open issues for bugs, questions, or suggestions.

---

## License

MIT License

---

## Contact

For security issues, please open a GitHub issue or contact the maintainers directly.
