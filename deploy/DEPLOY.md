1. Push into `main` and create tag and release with `NUMBER`
2. Launch [build-release.yml](../.github/workflows/build-release.yml) for release tag
3. Connect to server
4. Sync repo using root Dockerfile
5. Use envgen.py to actualize ENVS
6. Apply migrations for VALIDATOR and CLIENT using root Dockerfile
7. Use sync.py to download actual bins
8. Relaunch 
   - Client
   - Daemon
   - Oracle
   - Validator
