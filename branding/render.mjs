import sharp from 'sharp';
import { readFileSync } from 'fs';
const svg = readFileSync('branding/logo.svg');
await sharp(svg, { density: 300 })
  .resize(1024, 1024)
  .png()
  .toFile('branding/logo-1024.png');
console.log('rendered branding/logo-1024.png');
