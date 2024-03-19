import { Address, nativeToScVal, xdr } from 'stellar-sdk';
import { mintToken } from '../../mint_token.js';
import { AddressBook } from '../../utils/address_book.js';
import { getTokenBalance, invokeContract } from '../../utils/contract.js';
import { config } from '../../utils/env_config.js';
import { Token, TokensBook } from '../../utils/tokens_book.js';
import { getCurrentTimePlusOneHour } from '../../utils/tx.js';

const network = process.argv[2];
const loadedConfig = config(network);

export async function phoenixMultiAddLiquidity(numberOfPaths: number, tokensBook: TokensBook, addressBook: AddressBook) {
  const tokens = tokensBook.getTokensByNetwork(network);
  if(!tokens || tokens.length <= 0) throw new Error('No tokens found in the tokens book');

  try {
    // Generate paths
    const startAddress = tokens[0].address;
    const endAddress = tokens[1].address;

    const paths = generatePaths(tokens, startAddress, endAddress, numberOfPaths);

    for (let i = 0; i < paths.length; i++) {
      const path = paths[i];
      console.log('🚀 « path:', path);
      for (let i = 0; i < path.length - 1; i++) {
        const tokenA = path[i];
        const tokenB = path[i+1];
        
        // Mint tokens
        await mintToken(tokenA, 25000000000000, loadedConfig.admin.publicKey());
        await mintToken(tokenB, 25000000000000, loadedConfig.admin.publicKey());
        
        console.log('-------------------------------------------------------');
        console.log("Adding liquidity for pair: ", tokenA, "|", tokenB);
        console.log("TOKEN A Balance:", await getTokenBalance(tokenA, loadedConfig.admin.publicKey(), loadedConfig.admin));
        console.log("TOKEN B Balance:", await getTokenBalance(tokenB, loadedConfig.admin.publicKey(), loadedConfig.admin));
  
        // Add liquidity
        const addLiquidityParams: xdr.ScVal[] = [
          new Address(tokenA).toScVal(),
          new Address(tokenB).toScVal(),
          nativeToScVal(2000000000000, { type: 'i128' }),
          nativeToScVal(1250000000000, { type: 'i128' }),
          nativeToScVal(0, { type: 'i128' }),
          nativeToScVal(0, { type: 'i128' }),
          new Address(loadedConfig.admin.publicKey()).toScVal(),
          nativeToScVal(getCurrentTimePlusOneHour(), { type: 'u64' })
        ]
  
        await invokeContract('router', addressBook, 'add_liquidity', addLiquidityParams, loadedConfig.admin);
        console.log("TOKEN A Balance AFTER:", await getTokenBalance(tokenA, loadedConfig.admin.publicKey(), loadedConfig.admin));
        console.log("TOKEN B Balance AFTER:", await getTokenBalance(tokenB, loadedConfig.admin.publicKey(), loadedConfig.admin));
      }      
    }
  } catch (error) {
    console.log('🚀 « error:', error);
    
  }
}

function generatePaths(tokens: Token[], startAddress: string, endAddress: string, numberOfPaths: number): string[][] {
  // Filter out the start and end tokens from the list to avoid including them as intermediates
  const intermediateTokens = tokens.filter(token => token.address !== startAddress && token.address !== endAddress);

  // Function to generate a path
  const createPath = (intermediates: Token[]): string[] => {
    return [startAddress, ...intermediates.map(token => token.address), endAddress];
  };

  // Store generated paths
  let paths: string[][] = [];

  // Generate paths based on the number of paths requested
  for (let i = 0; i < numberOfPaths; i++) {
    // Determine the number of intermediates to include in this path
    const numIntermediates = Math.min(i, intermediateTokens.length);

    // Select intermediates for the path
    let selectedIntermediates: Token[] = [];
    for (let j = 0; j < numIntermediates; j++) {
      // Simple selection strategy: cycle through intermediates
      const intermediateIndex = (j + i) % intermediateTokens.length;
      selectedIntermediates.push(intermediateTokens[intermediateIndex]);
    }

    // Create and add the new path
    paths.push(createPath(selectedIntermediates));
  }

  return paths;
}
