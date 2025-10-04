import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NitroLegacyInventory } from "../target/types/nitro_legacy_inventory";

// This is only a skeleton to help you get started with Anchor tests.
// Run `anchor test` after updating program id in Anchor.toml and lib.rs.

describe("nitro-legacy-inventory", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.NitroLegacyInventory as Program<NitroLegacyInventory>;
  const authority = provider.wallet as anchor.Wallet;

  const [registryPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("nitro-registry"), authority.publicKey.toBuffer()],
    program.programId
  );

  it("initializes the registry", async () => {
    await program.methods
      .initializeRegistry()
      .accounts({
        registry: registryPda,
        authority: authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const registry = await program.account.inventoryRegistry.fetch(registryPda);
    console.log("Authority:", registry.authority.toBase58());
  });
});