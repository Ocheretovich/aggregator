// import {
//   BackstopClient,
//   EmitterClient,
//   Network,
//   PoolFactoryClient,
//   PoolInitMeta,
//   TxOptions,
// } from '@blend-capital/blend-sdk';
// import { Asset } from 'stellar-sdk';
// import { CometClient } from '../external/comet.js';
// import { tryDeployStellarAsset } from '../external/token.js';
import { AddressBook } from '../utils/address_book.js';
import { airdropAccount, bumpContractCode, installContract } from '../utils/contract.js';
import { config } from '../utils/env_config.js';
import { signWithKeypair } from '../utils/tx.js';

export async function deployAndInitContracts(addressBook: AddressBook) {
  const signWithAdmin = (txXdr: string) =>
    signWithKeypair(txXdr, rpc_network.passphrase, loadedConfig.admin);
  console.log('🚀 « signWithAdmin:', signWithAdmin);

  await airdropAccount(loadedConfig.admin);
  console.log('Installing Aggregator Contract');
  await installContract('aggregator', addressBook, loadedConfig.admin);
  await bumpContractCode('aggregator', addressBook, loadedConfig.admin);
  // add any other contracts here router / factory / etc same idea install and bump
  if (network != 'mainnet') {
    // mocks
    console.log('Installing and deploying: Phoenix Mocked Contracts');
    // await installContract('oraclemock', addressBook, config.admin);
    // await bumpContractCode('oraclemock', addressBook, config.admin);
    // await deployContract('oraclemock', 'oraclemock', addressBook, config.admin);
    // await bumpContractInstance('oraclemock', addressBook, config.admin);
    // // Tokens
    // console.log('Installing and deploying: Tokens');
    // await tryDeployStellarAsset(addressBook, config.admin, Asset.native());
    // await bumpContractInstance('XLM', addressBook, config.admin);
    // await tryDeployStellarAsset(
    //   addressBook,
    //   config.admin,
    //   new Asset('USDC', config.admin.publicKey())
    // );
    // await bumpContractInstance('USDC', addressBook, config.admin);
    // await tryDeployStellarAsset(
    //   addressBook,
    //   config.admin,
    //   new Asset('BLND', config.admin.publicKey())
    // );
    // await bumpContractInstance('BLND', addressBook, config.admin);
    // await tryDeployStellarAsset(
    //   addressBook,
    //   config.admin,
    //   new Asset('wETH', config.admin.publicKey())
    // );
    // await bumpContractInstance('wETH', addressBook, config.admin);
    // await tryDeployStellarAsset(
    //   addressBook,
    //   config.admin,
    //   new Asset('wBTC', config.admin.publicKey())
    // );
    // await bumpContractInstance('wBTC', addressBook, config.admin);
    // // Comet LP
    // await installContract('comet', addressBook, config.admin);
    // await bumpContractCode('comet', addressBook, config.admin);
    // await deployContract('comet', 'comet', addressBook, config.admin);
    // await bumpContractInstance('comet', addressBook, config.admin);
    // const comet = new CometClient(addressBook.getContractId('comet'));
    // await comet.init(config.admin.publicKey(), config.admin);
  }
  console.log('Deploying and Initializing Soroswap Aggregator');
  // await deployContract('emitter', 'emitter', addressBook, config.admin);
  // await bumpContractInstance('emitter', addressBook, config.admin);
  // const emitter = new EmitterClient(addressBook.getContractId('emitter'));
  // await deployContract('backstop', 'backstop', addressBook, config.admin);
  // await bumpContractInstance('backstop', addressBook, config.admin);
  // const backstop = new BackstopClient(addressBook.getContractId('backstop'));
  // await deployContract('poolFactory', 'poolFactory', addressBook, config.admin);
  // await bumpContractInstance('poolFactory', addressBook, config.admin);
  // const poolFactory = new PoolFactoryClient(addressBook.getContractId('poolFactory'));
  // await logInvocation(
  //   emitter.initialize(config.admin.publicKey(), signWithAdmin, rpc_network, tx_options, {
  //     blnd_token: addressBook.getContractId('BLND'),
  //     backstop: addressBook.getContractId('backstop'),
  //     backstop_token: addressBook.getContractId('comet'),
  //   })
  // );
  // await logInvocation(
  //   backstop.initialize(config.admin.publicKey(), signWithAdmin, rpc_network, tx_options, {
  //     backstop_token: addressBook.getContractId('comet'),
  //     emitter: addressBook.getContractId('emitter'),
  //     usdc_token: addressBook.getContractId('USDC'),
  //     blnd_token: addressBook.getContractId('BLND'),
  //     pool_factory: addressBook.getContractId('poolFactory'),
  //     drop_list: new Map(),
  //   })
  // );
  // const poolInitMeta: PoolInitMeta = {
  //   backstop: addressBook.getContractId('backstop'),
  //   blnd_id: addressBook.getContractId('BLND'),
  //   usdc_id: addressBook.getContractId('USDC'),
  //   pool_hash: Buffer.from(addressBook.getWasmHash('lendingPool'), 'hex'),
  // };
  // await logInvocation(
  //   poolFactory.initialize(
  //     config.admin.publicKey(),
  //     signWithAdmin,
  //     rpc_network,
  //     tx_options,
  //     poolInitMeta
  //   )
  // );
  // await bumpContractInstance('backstop', addressBook, config.admin);
  // await bumpContractInstance('emitter', addressBook, config.admin);
  // await bumpContractInstance('poolFactory', addressBook, config.admin);
}

const network = process.argv[2];
const addressBook = AddressBook.loadFromFile(network);

const loadedConfig = config(network);
const rpc_network = {
  rpc: loadedConfig.rpc.serverURL.toString(),
  passphrase: loadedConfig.passphrase,
  opts: { allowHttp: true },
};
// const tx_options: TxOptions = {
//   sim: false,
//   pollingInterval: 2000,
//   timeout: 30000,
//   builderOptions: {
//     fee: '10000',
//     timebounds: {
//       minTime: 0,
//       maxTime: 0,
//     },
//     networkPassphrase: config.passphrase,
//   },
// };
await deployAndInitContracts(addressBook);
// addressBook.writeToFile();
