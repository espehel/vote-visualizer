// openapi-typescript builds its output with the TypeScript compiler API, which the
// native TypeScript 7 compiler used by this repo does not provide. Give it its own
// TypeScript 5 instead of resolving its peer dependency to the workspace's TypeScript 7.
function readPackage(pkg) {
  if (pkg.name === "openapi-typescript") {
    delete pkg.peerDependencies?.typescript;
    pkg.dependencies = { ...pkg.dependencies, typescript: "5.9.3" };
  }
  return pkg;
}

module.exports = { hooks: { readPackage } };
