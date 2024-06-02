import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BondingCurve } from "../target/types/bonding_curve"
import { Connection, PublicKey, Keypair, SystemProgram, Transaction, sendAndConfirmTransaction, ComputeBudgetProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js"
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, getAssociatedTokenAddress } from "@solana/spl-token"
import { expect } from "chai";
import { BN } from "bn.js";
import keys from '../keys/users.json'
import key2 from '../keys/user2.json'
import { ASSOCIATED_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

// const connection = new Connection("https://api.devnet.solana.com")
const connection = new Connection("http://localhost:8899")
const curveSeed = "CurveConfiguration"
const POOL_SEED_PREFIX = "liquidity_pool"
const LIQUIDITY_SEED = "LiqudityProvider"
const SOL_VAULT_PREFIX = "liquidity_sol_vault"
function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

describe("bonding_curve", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.BondingCurve as Program<BondingCurve>;

  // custom setting 
  const user = Keypair.fromSecretKey(new Uint8Array(keys))
  const user2 = Keypair.fromSecretKey(new Uint8Array(key2))
  const tokenDecimal = 9
  const amount = new BN(1000000000).mul(new BN(10 ** tokenDecimal))

  let mint1: PublicKey
  let tokenAta1: PublicKey

  // let mint2: PublicKey
  // let tokenAta2: PublicKey

  console.log("Admin's wallet address is : ", user.publicKey.toBase58())

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

  // it("Airdrop to user wallet", async () => {
  //   console.log("Created a user, address is ", user2.publicKey.toBase58())
  //   console.log(`Requesting airdrop for another user ${user.publicKey.toBase58()}`)
  //   // 1 - Request Airdrop
  //   const signature = await connection.requestAirdrop(
  //     user2.publicKey,
  //     10 ** 9
  //   );
  //   // 2 - Fetch the latest blockhash
  //   const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
  //   // 3 - Confirm transaction success
  //   await connection.confirmTransaction({
  //     blockhash,
  //     lastValidBlockHeight,
  //     signature
  //   }, 'finalized');
  //   console.log("user balance : ", (await connection.getBalance(user.publicKey)) / 10 ** 9, "SOL")
  // })

  it("Mint token1 to user wallet", async () => {
    console.log("Trying to create and mint token1 to user's wallet")

    try {
      mint1 = await createMint(connection, user, user.publicKey, user.publicKey, tokenDecimal)
      console.log('mint1 address: ', mint1.toBase58());
      tokenAta1 = (await getOrCreateAssociatedTokenAccount(connection, user, mint1, user.publicKey)).address
      console.log('token1 account address: ', tokenAta1.toBase58());
      try {
        //minting 100 new tokens to the token address we just created
        await mintTo(connection, user, mint1, tokenAta1, user.publicKey, BigInt(amount.toString()))
      } catch (error) {
        console.log("🚀 ~ here:", error)
      }
      const tokenBalance = await connection.getTokenAccountBalance(tokenAta1)
      console.log("tokenBalance1 in user:", tokenBalance.value.uiAmount)
      console.log('token 1 successfully minted');
    } catch (error) {
      console.log("Token 1 creation error \n", error)
    }

  })

  // it("Mint token 2 to user wallet", async () => {
  //   console.log("Trying to create and mint token 2 to user's wallet")
  //   try {
  //     mint2 = await createMint(connection, user, user.publicKey, user.publicKey, tokenDecimal)
  //     console.log('mint 2 address: ', mint2.toBase58());

  //     tokenAta2 = (await getOrCreateAssociatedTokenAccount(connection, user, mint2, user.publicKey)).address
  //     console.log('token 2 account address: ', tokenAta2.toBase58());

  //     await mintTo(connection, user, mint2, tokenAta2, user.publicKey, BigInt(amount.toString()))
  //     const tokenBalance = await connection.getTokenAccountBalance(tokenAta2)
  //     console.log("token 2 Balance in user:", tokenBalance.value.uiAmount)
  //     console.log('token 2 successfully minted');
  //   } catch (error) {
  //     console.log("Token 2 creation error \n", error)
  //   }
  // })

  it("Initialize the contract", async () => {
    try {
      const [curveConfig] = PublicKey.findProgramAddressSync(
        [Buffer.from(curveSeed)],
        program.programId
      )
      const tx = new Transaction()
        .add(
          ComputeBudgetProgram.setComputeUnitLimit({ units: 10_000 }),
          ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1200_000 }),
          await program.methods
            .initialize(1)
            .accounts({
              dexConfigurationAccount: curveConfig,
              admin: user.publicKey,
              rent: SYSVAR_RENT_PUBKEY,
              systemProgram: SystemProgram.programId
            })
            .instruction()
        )
      tx.feePayer = user.publicKey
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
      console.log(await connection.simulateTransaction(tx))
      const sig = await sendAndConfirmTransaction(connection, tx, [user], { skipPreflight: true })
      console.log("Successfully initialized : ", `https://solscan.io/tx/${sig}?cluster=devnet`)
      let pool = await program.account.curveConfiguration.fetch(curveConfig)
      console.log("Pool State : ", pool)
    } catch (error) {
      console.log("Error in initialization :", error)
    }
  });

  it("create pool", async () => {
    try {
      const [poolPda] = PublicKey.findProgramAddressSync(
        [Buffer.from(POOL_SEED_PREFIX), mint1.toBuffer()],
        program.programId
      )
      const poolToken = await getAssociatedTokenAddress(
        mint1, poolPda, true
      )
      const tx = new Transaction()
        .add(
          ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
          ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }),
          await program.methods
            .createPool()
            .accounts({
              pool: poolPda,
              tokenMint: mint1,
              poolTokenAccount: poolToken,
              payer: user.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: SYSVAR_RENT_PUBKEY,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              systemProgram: SystemProgram.programId
            })
            .instruction()
        )
      tx.feePayer = user.publicKey
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
      console.log(await connection.simulateTransaction(tx))
      const sig = await sendAndConfirmTransaction(connection, tx, [user], { skipPreflight: true })
      console.log("Successfully created pool : ", `https://solscan.io/tx/${sig}?cluster=devnet`)
    } catch (error) {
      console.log("Error in creating pool", error)
    }
  })

  it("add liquidity", async () => {
    try {

      const [poolPda] = PublicKey.findProgramAddressSync(
        [Buffer.from(POOL_SEED_PREFIX), mint1.toBuffer()],
        program.programId
      )
      const [liquidityProviderAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from(LIQUIDITY_SEED), poolPda.toBuffer(), user.publicKey.toBuffer()],
        program.programId
      )

      const [poolSolVault] = PublicKey.findProgramAddressSync(
        [Buffer.from(SOL_VAULT_PREFIX), mint1.toBuffer()],
        program.programId
      )
      const poolToken = await getAssociatedTokenAddress(
        mint1, poolPda, true
      )

      const userAta1 = await getAssociatedTokenAddress(
        mint1, user.publicKey
      )

      const tx = new Transaction()
        .add(
          ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
          ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }),
          await program.methods
            .addLiquidity()
            .accounts({
              pool: poolPda,
              poolSolVault: poolSolVault,
              tokenMint: mint1,
              poolTokenAccount: poolToken,
              userTokenAccount: userAta1,
              user: user.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              rent: SYSVAR_RENT_PUBKEY,
              systemProgram: SystemProgram.programId
            })
            .instruction()
        )
      tx.feePayer = user.publicKey
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
      console.log(await connection.simulateTransaction(tx))
      const sig = await sendAndConfirmTransaction(connection, tx, [user], { skipPreflight: true })
      console.log("Successfully added liquidity : ", `https://solscan.io/tx/${sig}?cluster=devnet`)
      const userBalance = (await connection.getTokenAccountBalance(userAta1)).value.uiAmount
      const poolBalance = (await connection.getTokenAccountBalance(poolToken)).value.uiAmount
      console.log("after creating pool => userBalance:", userBalance)
      console.log("after creating pool => poolBalance:", poolBalance)
    } catch (error) {
      console.log("Error in adding liquidity", error)
    }
  })

  it("Buy token", async () => {
    try {
      const [curveConfig] = PublicKey.findProgramAddressSync(
        [Buffer.from(curveSeed)],
        program.programId
      )
      const [poolPda] = PublicKey.findProgramAddressSync(
        [Buffer.from(POOL_SEED_PREFIX), mint1.toBuffer()],
        program.programId
      )
      const poolToken = await getAssociatedTokenAddress(
        mint1, poolPda, true
      )
      const userAta1 = await getAssociatedTokenAddress(
        mint1, user.publicKey
      )
      const [poolSolVault] = PublicKey.findProgramAddressSync(
        [Buffer.from(SOL_VAULT_PREFIX), mint1.toBuffer()],
        program.programId
      )
      const tx = new Transaction()
        .add(
          ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
          ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }),
          await program.methods
            .buy(new BN(10 ** 8))
            .accounts({
              pool: poolPda,
              tokenMint: mint1,
              poolSolVault,
              poolTokenAccount: poolToken,
              userTokenAccount: userAta1,
              dexConfigurationAccount: curveConfig,
              user: user.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              rent: SYSVAR_RENT_PUBKEY,
              systemProgram: SystemProgram.programId
            })
            .instruction()
        )
      tx.feePayer = user.publicKey
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
      console.log(await connection.simulateTransaction(tx))
      const sig = await sendAndConfirmTransaction(connection, tx, [user], { skipPreflight: true })
      console.log("Successfully bought : ", `https://solscan.io/tx/${sig}?cluster=devnet`)

    } catch (error) {
      console.log("Error in buy transaction", error)
    }
  })

  it("Sell token", async () => {
    try {
      const [curveConfig] = PublicKey.findProgramAddressSync(
        [Buffer.from(curveSeed)],
        program.programId
      )
      const [poolPda] = PublicKey.findProgramAddressSync(
        [Buffer.from(POOL_SEED_PREFIX), mint1.toBuffer()],
        program.programId
      )
      const poolToken = await getAssociatedTokenAddress(
        mint1, poolPda, true
      )
      const userAta1 = await getAssociatedTokenAddress(
        mint1, user.publicKey
      )
      const [poolSolVault, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from(SOL_VAULT_PREFIX), mint1.toBuffer()],
        program.programId
      )
      const tx = new Transaction()
        .add(
          ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
          ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }),
          await program.methods
            .sell(amount.div(new BN(100)), bump)
            .accounts({
              pool: poolPda,
              tokenMint: mint1,
              poolSolVault,
              poolTokenAccount: poolToken,
              userTokenAccount: userAta1,
              dexConfigurationAccount: curveConfig,
              user: user.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              rent: SYSVAR_RENT_PUBKEY,
              systemProgram: SystemProgram.programId
            })
            .instruction()
        )
      tx.feePayer = user.publicKey
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
      console.log(await connection.simulateTransaction(tx))
      const sig = await sendAndConfirmTransaction(connection, tx, [user], { skipPreflight: true })
      console.log("Successfully Sold : ", `https://solscan.io/tx/${sig}?cluster=devnet`)

    } catch (error) {
      console.log("Error in sell transaction", error)
    }
  })


  it("Remove liquidity", async () => {
    try {

      const [poolPda] = PublicKey.findProgramAddressSync(
        [Buffer.from(POOL_SEED_PREFIX), mint1.toBuffer()],
        program.programId
      )

      const poolToken = await getAssociatedTokenAddress(
        mint1, poolPda, true
      )
      const userAta1 = await getAssociatedTokenAddress(
        mint1, user.publicKey
      )
      const [poolSolVault, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from(SOL_VAULT_PREFIX), mint1.toBuffer()],
        program.programId
      )

      const tx = new Transaction()
        .add(
          ComputeBudgetProgram.setComputeUnitLimit({ units: 200_000 }),
          ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }),
          await program.methods
            .removeLiquidity(bump)
            .accounts({
              pool: poolPda,
              tokenMint: mint1,
              poolTokenAccount: poolToken,
              userTokenAccount: userAta1,
              poolSolVault,
              user: user.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
              associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
              rent: SYSVAR_RENT_PUBKEY,
              systemProgram: SystemProgram.programId
            })
            .instruction()
        )
      tx.feePayer = user.publicKey
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
      console.log(await connection.simulateTransaction(tx))
      const sig = await sendAndConfirmTransaction(connection, tx, [user], { skipPreflight: true })
      console.log("Successfully added liquidity : ", `https://solscan.io/tx/${sig}?cluster=devnet`)
    } catch (error) {
      console.log("Error in removing liquidity", error)
    }
  })
});


