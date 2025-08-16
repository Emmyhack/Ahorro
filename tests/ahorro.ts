import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Ahorro } from "../target/types/ahorro";

describe("ahorro", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Ahorro as Program<Ahorro>;

  it.skip("creates a group (pending localnet + USDC mint setup)", async () => {
    // TODO: create test USDC mint and call create_group
    // await program.methods.createGroup(/* args */).rpc();
  });
});
