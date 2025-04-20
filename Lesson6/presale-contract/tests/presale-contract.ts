import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PresaleContract } from "../target/types/presale_contract";
import { web3 } from "@coral-xyz/anchor";
import { BN } from "bn.js";
import { assert } from "chai";
import { readFileSync } from "fs";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, createMint, mintTo, createAssociatedTokenAccount, 
  getOrCreateAssociatedTokenAccount, getAccount } from "@solana/spl-token";
import { log } from "console";
import { token } from "@coral-xyz/anchor/dist/cjs/utils";

const DECIMAL = 1;

describe("presale-contract", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PresaleContract as Program<PresaleContract>;
  const authority = loadLocalSigner('id.json');
  const mint = new web3.PublicKey("Di2VkF6679HfuS8HjgpMESjoQpHY7f4zM7ELiAMsDTP6")
  
  let authorityAta: PublicKey;
  let user: Keypair;
  let userAta: PublicKey;
  let vaultAta: PublicKey;
  let presaleContract: PublicKey;
  let vaultSignerBump: number;
  let vaultSigner: PublicKey;
  let userPurchase: PublicKey;

  before(async () => {
    
    const authorityAtaAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority,
      mint,
      authority.publicKey
    );
    authorityAta = authorityAtaAccount.address;
    console.log("Authority ATA : " + authorityAta);

    user = authority as Keypair;
    const userAtaAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user,
      mint,
      user.publicKey
    );
    userAta = userAtaAccount.address;
    console.log("User ATA : " + userAta);
    

    const [presalePDA, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("presale-event"), mint.toBuffer()],
      program.programId
    );
    presaleContract = presalePDA;
    vaultSignerBump = bump;
    vaultSigner = presaleContract; // vault PDA signer is vaultState
 
    console.log("Presale PDA :" + presaleContract);
    

    let vaultAtaAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      authority,
      mint, 
      vaultSigner,
      true
    );
    vaultAta = vaultAtaAccount.address;

    console.log("vault ATA : " + vaultAta);
    

    const [userPurchasePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user-purchase"), user.publicKey.toBuffer(), mint.toBuffer()],
      program.programId
    );
    userPurchase = userPurchasePDA;
    console.log("user purchase PDA: " + userPurchase);
  });

  it("Setup presale event", async () => {
    const now = Math.floor(Date.now() / 1000);
    const startPresale = now;
    const endPresale = startPresale + 2*3600;
    const tgeTs = startPresale + 0;
    const pricePerToken = new anchor.BN(0.1 * LAMPORTS_PER_SOL); // 0.1 SOL
  
    await program.methods
      .setupPresale(
        new BN(startPresale),
        new BN(endPresale),
        new anchor.BN(tgeTs),
        pricePerToken
      )
      .accounts({
        presaleEvent: presaleContract,
        tokenMint: mint,
        signer: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

      const presale_contract_data = await program.account.presaleEvent.fetch(presaleContract);
      console.log("Total token : " + presale_contract_data.totalTokens);
  });

  it("Provide token to vault", async () => {
    await program.methods.provideTokenToVault(new BN(50*DECIMAL))
      .accounts({
        presaleEvent: presaleContract,
        vaultAta: vaultAta,
        authorityAta: authorityAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

      const presale_contract_data = await program.account.presaleEvent.fetch(presaleContract);
      console.log("Total token : " + presale_contract_data.totalTokens);
      
  });

  it("purchase token by sol", async () => {
    const amountToBuy = new BN(10); // 10 token
    const pricePerToken = new BN(0.1 * LAMPORTS_PER_SOL); // = 0.1 SOL
    const totalPrice = amountToBuy.mul(pricePerToken); // 50 * 0.1 SOL

    console.log("user" + user);
    console.log("presale event PDA " + presaleContract);
    console.log("user purchase PDA " + userPurchase);

    const presale_contract_data = await program.account.presaleEvent.fetch(presaleContract);
    console.log("Total token : " + presale_contract_data.totalTokens);
    
    await program.methods
      .purchaseTokenBySol(amountToBuy)
      .accounts({
        presaleEvent: presaleContract,
        userPurchase: userPurchase,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
  
    const user_purchase_data = await program.account.userPurchase.fetch(userPurchase);
    console.log("✅ Claimed amount : {}", user_purchase_data.claimedTokens);
  });
  

  it("claim token", async () => {
    await program.methods
    .claimToken()
    .accounts({
      presaleEvent: presaleContract,
      userPurchase: userPurchase,
      vaultAta: vaultAta,
      userAta: userAta,
      tokenProgram: TOKEN_PROGRAM_ID,
      user: user.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([user])
    .rpc();


    const user_purchase_data = await program.account.userPurchase.fetch(userPurchase);
    console.log("❌ unclaimed amount" + user_purchase_data.totalPurchasedToken);
    console.log("✅ claimed amount" + user_purchase_data.claimedTokens);
  });

  it("withdraw sol from vault", async () => {
    await program.methods
      .withdrawSolFromVault()
      .accounts({
        vault: presaleContract,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();
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
