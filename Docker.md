Build the container:

```
docker build .
```

Use the sha256 image to test it:

```
docker run -v ~/RustroverProjects/version-bump/tests:/tests/ -e INPUT_VERSION=1.2.3 -e INPUT_FILE_PATH=/tests/test.csproj e99854ac604244de19f3473a72079c1996318cf9874b12280ee00082bad415f5
```
