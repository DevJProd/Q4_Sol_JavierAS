import wallet from "../wba-wallet.json"
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import { 
    createMetadataAccountV3, 
    CreateMetadataAccountV3InstructionAccounts, 
    CreateMetadataAccountV3InstructionArgs,
    DataV2Args
} from "@metaplex-foundation/mpl-token-metadata";
import { createSignerFromKeypair, signerIdentity, publicKey } from "@metaplex-foundation/umi";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

// Define our Mint address
const mint = publicKey("7BdfrUb2t8DpAeKggPB4ygGRaTBYygGQ2E4rNgEaPW8K")

// Create a UMI connection
const umi = createUmi('https://api.devnet.solana.com');
const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(createSignerFromKeypair(umi, keypair)));

(async () => {
    try {
        // Start here
        let accounts: CreateMetadataAccountV3InstructionAccounts = {
            payer:signer,
            updateAuthority:signer,
            mint: mint,
            mintAuthority: signer
        }

        let data: DataV2Args = {
            name:"DEVJ Coin",
            symbol:"DEVJ",
            uri:"https://apricot-urban-horse-486.mypinata.cloud/ipfs/QmWRVDLgJngk1Q8t5GYhKi5KMaNb2VSFc6JGAWthwW6woX",
            sellerFeeBasisPoints:6,
            creators:null,
            collection:null,
            uses:null
        }

        let args: CreateMetadataAccountV3InstructionArgs = {
            data: data,
            isMutable: false,
            collectionDetails: null
        }

        let tx = createMetadataAccountV3(
             umi,
             {
                ...accounts,
                ...args
             }
        )

        let result = await tx.sendAndConfirm(umi);
        console.log(bs58.encode(result.signature));
    } catch(e) {
        console.error(`Oops, something went wrong: ${e}`)
    }
})();
