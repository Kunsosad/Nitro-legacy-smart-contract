import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("Deploying with:", deployer.address);

  const contractFactory = await ethers.getContractFactory("NitroLegacyInventory");
  const contract = await contractFactory.deploy();
  await contract.waitForDeployment();

  console.log("NitroLegacyInventory deployed to:", await contract.getAddress());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});