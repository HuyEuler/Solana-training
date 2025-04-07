import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MyProgram2 } from "../target/types/my_program2";
import { BN } from "bn.js";

describe("my-program2", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.MyProgram2 as Program<MyProgram2>;

  it("Is initialized!", async () => {
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Caculate sum", async() => {
    const tx = await program.methods.sum(new BN(1), new BN(3)).rpc();
    console.log("Your transaction signature", tx);
  });
});