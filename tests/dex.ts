import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BondingCurve } from "../target/types/dex";
import { expect } from "chai";

describe("bonding_curve", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.BondingCurve as Program<BondingCurve>;

  it("Liquidity pool created", async () => {
    
    const [poolPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_pool")],
      program.programId
    )

    await program.methods.initializeLiquidityPool().accounts({}).rpc();

    let pool = await program.account.liquidityPool.fetch(poolPda);

    expect(pool.assets.length).to.equal(0);
    expect(pool.bump).not.null;
  });
});
