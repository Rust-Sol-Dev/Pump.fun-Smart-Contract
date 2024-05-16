import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BondingCurveJ } from "../target/types/bonding_curve";
import keys from '../keys/users.json'
import key2 from '../keys/user2.json'
import { Connection, Keypair, PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccountInstruction, createMint, getAssociatedTokenAddress, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { BN } from "bn.js";

// const connection = new Connection("https://white-proportionate-putty.solana-devnet.quiknode.pro/11132715a936f8adb03c940c627d6c0b9369d9e6/")
const connection = new Connection("http://localhost:8899")
const GLOBAL_SEED = "GLOBAL_SEED"
const LOCK_SEED = "LOCK_SEED"


export function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

describe("bonding_curve", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());


  // custom setting 
  const user = Keypair.fromSecretKey(new Uint8Array(keys))
  const user2 = Keypair.fromSecretKey(new Uint8Array(key2))
  let mint: PublicKey
  let tokenAta: PublicKey
  const tokenDecimal = 9
  const amount = new BN(1000000000).mul(new BN(10 ** tokenDecimal))
  const poolId = new PublicKey("5cvj5rEEocG5Wvh3oJuY6MoYj7gVZd8aoXSLZjDY6W4W") // actually it's token mint

  const lockDay = 2
  console.log("Admin's wallet address is : ", user.publicKey.toBase58())

  const program = anchor.workspace.BondingCurveJ as Program<BondingCurveJ>;
  it("Airdrop to admin wallet", async () => {
    console.log(`Requesting airdrop to admin for 1SOL : ${user.publicKey.toBase58()}`)
    // 1 - Request Airdrop
    const signature = await connection.requestAirdrop(
      user.publicKey,
      10 ** 9
    );
    // 2 - Fetch the latest blockhash
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
    // 3 - Confirm transaction success
    await connection.confirmTransaction({
      blockhash,
      lastValidBlockHeight,
      signature
    }, 'finalized');
    console.log("admin wallet balance : ", (await connection.getBalance(user.publicKey)) / 10 ** 9, "SOL")
  })

  it("Airdrop to user wallet", async () => {
    console.log("Created a user, address is ", user2.publicKey.toBase58())
    console.log(`Requesting airdrop for another user ${user.publicKey.toBase58()}`)
    // 1 - Request Airdrop
    const signature = await connection.requestAirdrop(
      user2.publicKey,
      10 ** 9
    );
    // 2 - Fetch the latest blockhash
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
    // 3 - Confirm transaction success
    await connection.confirmTransaction({
      blockhash,
      lastValidBlockHeight,
      signature
    }, 'finalized');
    console.log("user balance : ", (await connection.getBalance(user.publicKey)) / 10 ** 9, "SOL")
  })

  it("Mint token to user wallet", async () => {
    console.log("Trying to reate and mint token to user's wallet")
    console.log("Here, contract uses this token as LP token")
    //create mint
    try {
      mint = await createMint(connection, user, user.publicKey, user.publicKey, tokenDecimal)
      console.log('mint address: ' + mint.toBase58());

      tokenAta = (await getOrCreateAssociatedTokenAccount(connection, user, mint, user.publicKey)).address
      console.log('token account address: ' + tokenAta.toBase58());

      //minting 100 new tokens to the token address we just created
      await mintTo(connection, user, mint, tokenAta, user.publicKey, BigInt(amount.toString()))
      const tokenBalance = await connection.getTokenAccountBalance(tokenAta)
      tokenBalance.value.uiAmount
      console.log("tokenBalance in user:", tokenBalance.value.uiAmount)
      console.log('token successfully minted');
    } catch (error) {
      console.log("Token creation error \n", error)
    }
  })

  it("Is initialized!", async () => {
    console.log("Admin initializes the smart contract")
    try {
      const [globalState] = PublicKey.findProgramAddressSync(
        [Buffer.from(GLOBAL_SEED)],
        program.programId
      )

      const tx = await program.methods.initialize()
        .accounts({
          admin: user.publicKey,
          globalState,
          systemProgram: SystemProgram.programId
        })
        .signers([user])
        .transaction()
      tx.feePayer = user.publicKey
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
      console.log(await connection.simulateTransaction(tx))
      await sendAndConfirmTransaction(connection, tx, [user])
      console.log("Below is global state value")
      const globalStateValue = await program.account.globalState.fetch(globalState)
      console.log("globalStateValue: \n", globalStateValue)
    } catch (error) {
      console.log("error in initialization :", error)
    }
  })

});



