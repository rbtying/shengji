const ManifestPlugin = require('webpack-manifest-plugin');

module.exports = {
  webpack: (config, env) => {
    if (env === 'production') {
      //JS Overrides
      config.output.filename = 'main.js';
      delete config.output.chunkFilename;
      config.optimization.splitChunks = {
        cacheGroups: {
          default: false,
        },
      };
      config.optimization.runtimeChunk = false;

      //CSS Overrides
      config.plugins[4].filename = 'dist/css/[name].css';

      // Remove ManifestPlugin
      config.plugins.splice(5, 5);
    }

    return config;
  },
};
