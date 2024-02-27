import ethSigUtil from 'eth-sig-util';

const account = '0x521ec61eb00a45fa2a17e92762dd1d43de9ffe26';
const data = '0x68656c6c6f'; // hello
const sig = '0x7c4f5f867466a9244d9c5e59d7b0f0a57b56209789f13997cff86485875eb6bc2e7e802ed7c05c6dc6f5bf27735e43a0240d3404571fbbb222c769f56488e77f1c';

const recovered = ethSigUtil.recoverPersonalSignature({
    data,
    sig,
});

console.log('account', account);
console.log('recovered', recovered); // 0x521ec61eb00a45fa2a17e92762dd1d43de9ffe26
