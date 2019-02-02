// webpack.config.js
const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const prod = process.env.NODE_ENV === 'production';

const browserConfig = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  plugins: [
    new HtmlWebpackPlugin({
      title: "Getting started with WASM",
      template: 'template.html'
    })
  ],
	mode: prod ? 'production' : 'development'
};

const workerConfig = {
  entry: "./worker.js",
  target: 'webworker',
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "worker.js",
  },
	mode: prod ? 'production' : 'development'
};

module.exports = [browserConfig, workerConfig];
