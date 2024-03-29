import './style.css';
import { ethers } from 'ethers';

const sign = async () => {
  console.log('signing...');
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

  // Convert the signature string to a Uint8Array
  console.log('posting some bytes bby');

  fetch(window.location.href, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ signature: sig }),
  })
    .then(response => {
      if (response.ok) {
        console.log('Success: Signature accepted');
      } else if (response.status === 400) {
        console.error('Error: Bad request');
      } else {
        console.error('Error: Unexpected response status', response.status);
      }
    })
    .catch((error) => console.error('Network error:', error));

};

const app = document.getElementById('app');

const signButton = document.createElement('button');
signButton.onclick = sign;
signButton.innerHTML = 'Sign Hello message to confirm token ownership';
app.appendChild(signButton);
