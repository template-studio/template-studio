/**
 * CodeGen Studio 图标生成脚本
 *
 * 使用方法：
 * 1. 安装依赖：npm install sharp --save-dev
 * 2. 运行脚本：node scripts/generate-icons.js
 *
 * 或者使用在线工具：
 * - https://realfavicongenerator.net/
 * - https://www.xiconeditor.com/
 * - https://iconifier.net/
 */

const fs = require('fs');
const path = require('path');

// 检查是否安装了 sharp
let sharp;
try {
  sharp = require('sharp');
} catch (e) {
  console.log('请先安装 sharp: npm install sharp --save-dev');
  console.log('或者使用在线工具转换图标：');
  console.log('  - https://realfavicongenerator.net/');
  console.log('  - https://www.xiconeditor.com/');
  process.exit(1);
}

const ICONS_DIR = path.join(__dirname, '../src-tauri/icons');
const SVG_FILE = path.join(ICONS_DIR, 'icon.svg');

// 需要生成的图标尺寸
const ICONS = [
  { name: '32x32.png', size: 32 },
  { name: '128x128.png', size: 128 },
  { name: '128x128@2x.png', size: 256 },
  { name: '256x256.png', size: 256 },
  { name: '512x512.png', size: 512 },
];

async function generateIcons() {
  console.log('🚀 开始生成 CodeGen Studio 图标...\n');

  // 检查 SVG 文件是否存在
  if (!fs.existsSync(SVG_FILE)) {
    console.error('❌ 找不到 SVG 文件:', SVG_FILE);
    process.exit(1);
  }

  // 生成 PNG 图标
  for (const icon of ICONS) {
    const outputPath = path.join(ICONS_DIR, icon.name);
    try {
      await sharp(SVG_FILE)
        .resize(icon.size, icon.size)
        .png()
        .toFile(outputPath);
      console.log(`✅ 生成: ${icon.name} (${icon.size}x${icon.size})`);
    } catch (err) {
      console.error(`❌ 生成 ${icon.name} 失败:`, err.message);
    }
  }

  // 生成 ICO (Windows)
  try {
    const icoBuffer = await sharp(SVG_FILE)
      .resize(256, 256)
      .png()
      .toBuffer();

    // 使用 png-to-ico 或者手动说明
    console.log('\n⚠️  ICO 文件需要额外工具生成：');
    console.log('   推荐使用: https://www.xiconeditor.com/');
    console.log('   上传 512x512.png 即可生成 icon.ico');
  } catch (err) {
    console.error('❌ 准备 ICO 失败:', err.message);
  }

  // ICNS (macOS) 说明
  console.log('\n⚠️  ICNS 文件需要额外工具生成：');
  console.log('   macOS: 使用 iconutil 命令');
  console.log('   或在线工具: https://iconifier.net/');

  console.log('\n🎉 图标生成完成！');
  console.log('\n📁 输出目录:', ICONS_DIR);
}

generateIcons().catch(console.error);
