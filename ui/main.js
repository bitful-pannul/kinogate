import './style.css';
import { ethers } from 'ethers';

const sign = async () => {
  const message = 'hello';
  const hexMessage = ethers.utils.hexlify(ethers.utils.toUtf8Bytes(message));
  const web3Provider = new ethers.providers.Web3Provider(window.ethereum, 'any');
  const [account] = await web3Provider.send('eth_requestAccounts');
  const sig = await web3Provider.getSigner().signMessage(message);
  const recovered = ethers.utils.verifyMessage(message, sig);

  console.log('message', message);
  console.log('hexMessage', hexMessage);
  console.log('account', account);
  console.log('sig', sig);
  console.log('recovered', recovered);
};

const app = document.getElementById('app');

const signButton = document.createElement('button');
signButton.onclick = sign;
signButton.innerHTML = 'Sign Hello message to confirm token ownership';
app.appendChild(signButton);
