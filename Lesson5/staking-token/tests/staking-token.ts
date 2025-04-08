import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StakingToken } from "../target/types/staking_token";
import { web3 } from "@coral-xyz/anchor";
import { BN } from "bn.js";
import { assert } from "chai";
import { readFileSync } from "fs";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, createMint, mintTo, createAssociatedTokenAccount, 
  getOrCreateAssociatedTokenAccount, getAccount } from "@solana/spl-token";


// const LAMPORTS_PER_SOL = 1_000_000_000;


describe("staking_token", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StakingToken as Program<StakingToken>;

  const admin = loadLocalSigner('id.json');
  const mint = new web3.PublicKey("8dh63X7teT3h1SLyTSwchfYRRWncer18mFe1X23RsbSr")
  
  let user: Keypair;
  let anotherUser: Keypair;
  // let mint: PublicKey;
  let userAta: PublicKey;
  let vaultAta: PublicKey;
  let vaultState: PublicKey;
  let vaultSignerBump: number;
  let vaultSigner: PublicKey;
  let stakeAccount: PublicKey;

  before(async () => {
    user = admin as Keypair;
    anotherUser = loadLocalSigner('wallet-tmp.json') as Keypair;
    // mint = await createMint(
    //   provider.connection,
    //   admin,
    //   admin.publicKey,
    //   null,
    //   0,
    // );

    const userAtaAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user,
      mint,
      user.publicKey
    );
    userAta = userAtaAccount.address;

    [vaultState, vaultSignerBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), admin.publicKey.toBuffer()],
      program.programId
    );
    vaultSigner = vaultState; // vault PDA signer is vaultState

    let vaultAtaAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin,
      mint, 
      vaultSigner,
      true
    );
    vaultAta = vaultAtaAccount.address;

    const [stakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("data"), user.publicKey.toBuffer()],
      program.programId
    );
    stakeAccount = stakePda;

    // await mintTo(
    //   provider.connection,
    //   admin,
    //   mint,
    //   userAta,
    //   admin,
    //   1_000_000_000 // 1000 tokens
    // );
  });

  it("Initialize vault", async () => {
    await program.methods.initializeVault()
      .accounts({
        vaultState,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

    console.log("Vault address : " + vaultState);
  });

  it("Update rps", async() => {
    await program.methods.updateRps()
      .accounts({
        vaultState: vaultState,
        vault: vaultState,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin])
      .rpc();

      const vault_state = await program.account.vaultState.fetch(vaultState);
      console.log("Total reward : " + vault_state.totalReward);
      console.log("RPS : " + vault_state.rps);
  });

  it("User can stake tokens", async () => {
    console.log("Vault ATA address : " + vaultAta);
    
    await program.methods.stake(new anchor.BN(100))
      .accounts({
        vaultState,
        stakeAccount,
        user: user.publicKey,
        userAta,
        vaultAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([user])
      .rpc();

    const stakeAccountData = await program.account.stakeAccount.fetch(stakeAccount);
    console.log("Pending reward : " + stakeAccountData.pendingReward);
    
  });

  it("User can claim reward", async () => {
    const stakeAccountData = await program.account.stakeAccount.fetch(stakeAccount);
    console.log("Pending reward before withdraw: " + stakeAccountData.pendingReward);

    await program.methods.claimReward()
      .accounts({
        stakeAccount,
        vaultState,
        vault: vaultState,
        user: user.publicKey,
        systemProgram: SystemProgram.programId
      })
      .signers([user])
      .rpc();

    const stakeAccountData2 = await program.account.stakeAccount.fetch(stakeAccount);
    console.log("Pending reward after withdraw: " + stakeAccountData2.pendingReward);
    // assert.equal(stakeAccountData2.pendingReward.toNumber(), 0);
  });

  it("User can unstake tokens", async () => {
    const userAtaBefore = await getAccount(provider.connection, userAta);
    const stakeAccountData = await program.account.stakeAccount.fetch(stakeAccount);
    console.log("Amount token in vault before unstake: " + stakeAccountData.amountToken);

    await program.methods.unstake(new anchor.BN(50))
      .accounts({
        vaultState,
        stakeAccount,
        user: user.publicKey,
        userAta,
        vaultAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([user])
      .rpc();

    const stakeAccountData2 = await program.account.stakeAccount.fetch(stakeAccount);
    console.log("Amount token in vault after unstake: " + stakeAccountData2.amountToken);
    
    assert.equal(stakeAccountData2.amountToken.toNumber(), 0);

    const userAtaAfter = await getAccount(provider.connection, userAta);
    assert.isAbove(Number(userAtaAfter.amount), Number(userAtaBefore.amount));
  });
});



interface Signer {
  publicKey: PublicKey;
  secretKey: Uint8Array;
}

function loadLocalSigner(path: String): Signer {
  const keypairPath = `${process.env.HOME}/.config/solana/${path}`;
  const secretKeyString = readFileSync(keypairPath, { encoding: "utf-8" });
  const secretKeyArray = JSON.parse(secretKeyString) as number[];
  const keypair = Keypair.fromSecretKey(new Uint8Array(secretKeyArray));

  return {
      publicKey: keypair.publicKey,
      secretKey: keypair.secretKey,
  };
}