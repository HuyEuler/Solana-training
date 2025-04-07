import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BankApp } from "../target/types/bank_app";
import { assert } from "chai";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";
import { readFileSync } from "fs";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("bank-app", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BankApp as Program<BankApp>;
  
  const user = loadLocalSigner('id.json');
  const anotherUser = loadLocalSigner('wallet-tmp.json');

  let bankAccount : anchor.web3.PublicKey, bankAccountBump: number;

  before(async() => {
    [bankAccount, bankAccountBump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("bank_acc"), user.publicKey.toBuffer()],
      program.programId
    );
  })
  
  console.log("Bank Account PDA: " + bankAccount);
  console.log("User " + user.publicKey);

  it("Initializes a bank account", async () => {
    await program.methods.initializeAccount()
      .accounts({
        bankAccount: bankAccount,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

      console.log("PDA address : " + bankAccount)
  });

  it("Deposits SOL", async () => {
    const depositAmount = new anchor.BN(1 * LAMPORTS_PER_SOL); // 1 SOL

    await program.methods.deposit(depositAmount)
      .accounts({
        bankAccount: bankAccount,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const balance = await program.methods.getBalance()
      .accounts({ bankAccount: bankAccount })
      .view();
    console.log("My balance after deposit: " + balance);
  });

  it("Withdraws SOL", async () => {
    const withdrawAmount = new anchor.BN(0.5 * LAMPORTS_PER_SOL); // 0.5 SOL

    console.log("My PDA : " + bankAccount);
    await program.methods.withdraw(withdrawAmount)
      .accounts({
        bankAccount: bankAccount,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const balance = await program.methods.getBalance()
      .accounts({ bankAccount: bankAccount })
      .view();
    
    console.log("My balance after withdraw: " + balance);
  });

  it("Should fail while another try to withdraw", async () => {
    const withdrawAmount = new anchor.BN(0.5 * LAMPORTS_PER_SOL); // 0.5 SOL

    console.log("another user: " + anotherUser.publicKey);

    await program.methods.withdraw(withdrawAmount)
      .accounts({
        bankAccount: bankAccount,
        user: anotherUser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([anotherUser])
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