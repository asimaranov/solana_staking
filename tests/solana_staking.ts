import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SolanaStaking } from "../target/types/solana_staking";
import { Account, createMint, getAccount, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { utf8 } from "@project-serum/anchor/dist/cjs/utils/bytes";
import { expect } from "chai";
import { BN } from "bn.js";

describe("solana_staking", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SolanaStaking as Program<SolanaStaking>;
  const owner = (program.provider as anchor.AnchorProvider).wallet;
  const payer = anchor.web3.Keypair.generate();


  const testRoundTime = new anchor.BN(60*60*24);
  let fctrMint: anchor.web3.PublicKey;
  let bcdevMint: anchor.web3.PublicKey;

  let stakingFctrAccount: Account;
  let stakingBcdevAccount: Account;

  let stakingPda: anchor.web3.PublicKey;
  const ONE_FCTR = new BN(10).pow(new BN(12));


  it("Is initialized!", async () => {
    await program.provider.connection.confirmTransaction(await program.provider.connection.requestAirdrop(payer.publicKey, 1000000 * anchor.web3.LAMPORTS_PER_SOL));

    [stakingPda, ] = await anchor.web3.PublicKey.findProgramAddress([utf8.encode("staking")], program.programId);
    
    fctrMint = await createMint(program.provider.connection, payer, stakingPda, null, 12);
    bcdevMint = await createMint(program.provider.connection, payer, stakingPda, null, 18);

    stakingFctrAccount = await getOrCreateAssociatedTokenAccount(program.provider.connection, payer, fctrMint, stakingPda, true);

    const tx = await program.methods.initialize(testRoundTime, fctrMint, bcdevMint).accounts({
      staking: stakingPda,
      stakingFctrAccount: stakingFctrAccount.address,
      owner: owner.publicKey,
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Test user registration", async () => {
    const [stakerInfo, ] = await anchor.web3.PublicKey.findProgramAddress([utf8.encode("staker-info"), owner.publicKey.toBuffer()], program.programId);

    await program.methods.register().accounts({
      staker: owner.publicKey,
      stakerInfo: stakerInfo
    }).rpc()
  })

  it("Test fctr buying", async () => {
    const testAmount = new BN(10).mul(ONE_FCTR);

    const [stakerInfo, ] = await anchor.web3.PublicKey.findProgramAddress([utf8.encode("staker-info"), owner.publicKey.toBuffer()], program.programId);
    let userFctrAccount = await getOrCreateAssociatedTokenAccount(program.provider.connection, payer, fctrMint, owner.publicKey);
    const lampBalanceBeforeStake = await program.provider.connection.getBalance(owner.publicKey);
    const fctrBalanceBeforeStake = await userFctrAccount.amount;
    await program.methods.buyFctr(testAmount).accounts({
      staking: stakingPda,
      fctrMint: fctrMint,
      user: owner.publicKey,
      stakerInfo: stakerInfo,
      userFctrAccount: userFctrAccount.address
    }).rpc();

    userFctrAccount = await getAccount(program.provider.connection, userFctrAccount.address);

    const lampBalanceAfterStake = await program.provider.connection.getBalance(owner.publicKey);
    const lampToTake = testAmount.mul(new BN(anchor.web3.LAMPORTS_PER_SOL)).div(ONE_FCTR).div(new BN(109))
    console.log(`Exchanging ${testAmount} fctr tokens for ${lampToTake} lamports`)
    expect(lampBalanceAfterStake).lte(lampBalanceBeforeStake - lampToTake.toNumber());
    expect(userFctrAccount.amount - fctrBalanceBeforeStake >= BigInt(testAmount.toString(10))).to.be.true;
  })
});
