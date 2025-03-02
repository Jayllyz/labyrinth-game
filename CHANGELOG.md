# Changelog

All notable changes to this project will be documented in this file.

## [1.0.3](https://github.com/Jayllyz/labyrinth-game/compare/v1.0.2..v1.0.3) - 2025-03-02

### 📇 Features

- *(client)* Implement terminal user interface ([#66](https://github.com/Jayllyz/labyrinth-game/issues/66)) - ([8b3e618](https://github.com/Jayllyz/labyrinth-game/commit/8b3e6182e5f1c3fda15bc5a0f7d4a801a896adb7))
- *(instructions)* Add ObjectiveAlly cell type and update solver ([#77](https://github.com/Jayllyz/labyrinth-game/issues/77)) - ([08ec797](https://github.com/Jayllyz/labyrinth-game/commit/08ec797f43d5a44a81d61cb70b2e2e8fc54ce33f))
- *(server)* Implement mini server ([#88](https://github.com/Jayllyz/labyrinth-game/issues/88)) - ([c141f6d](https://github.com/Jayllyz/labyrinth-game/commit/c141f6d145ef5b0d9d6757c7a8fe35e2e971354d))
- Args for algorithm ([#75](https://github.com/Jayllyz/labyrinth-game/issues/75)) - ([402ffb4](https://github.com/Jayllyz/labyrinth-game/commit/402ffb4813799cd6267d6af41e9b8171cc88e1f4))
- Alian algorithm ([#74](https://github.com/Jayllyz/labyrinth-game/issues/74)) - ([ac6ca7d](https://github.com/Jayllyz/labyrinth-game/commit/ac6ca7d535bf42a46707e43fdd95506106e84eaa))
- Implement custom GameError  ([#59](https://github.com/Jayllyz/labyrinth-game/issues/59)) - ([fc7476a](https://github.com/Jayllyz/labyrinth-game/commit/fc7476a8722bc446a8df35a626f7e70e3e96de89))
- Tremeaux algorithm ([#58](https://github.com/Jayllyz/labyrinth-game/issues/58)) - ([b00fcaa](https://github.com/Jayllyz/labyrinth-game/commit/b00fcaa5071062c06482e98ae05b8d0dacc9f475))
- Implement graphs for maze resolution ([#56](https://github.com/Jayllyz/labyrinth-game/issues/56)) - ([c9bcefa](https://github.com/Jayllyz/labyrinth-game/commit/c9bcefa8e30031ccb79ab063c47c9e029f6b0178))

### 🐛 Bug Fixes

- *(deps)* Update rust crate criterion2 to v3 ([#90](https://github.com/Jayllyz/labyrinth-game/issues/90)) - ([2f9ff0b](https://github.com/Jayllyz/labyrinth-game/commit/2f9ff0b916cd930c027211929ef5753da298e05d))
- *(tui)* Handle bounds calculations safely and split in small functions ([#76](https://github.com/Jayllyz/labyrinth-game/issues/76)) - ([3b6e366](https://github.com/Jayllyz/labyrinth-game/commit/3b6e366c7a3cc686e9f2326f3ad4db56457cdb4f))
- Lint issue - ([294c673](https://github.com/Jayllyz/labyrinth-game/commit/294c67354a76391adc99d8b4d3c0fd903a9f0aff))
- Fix tremeaux ([#60](https://github.com/Jayllyz/labyrinth-game/issues/60)) - ([e712552](https://github.com/Jayllyz/labyrinth-game/commit/e7125522bbd1e0c22d7bc4f4535e3a3152ab6d02))

### 🚜 Refactor

- *(client)* Regroup arguments in structures ([#83](https://github.com/Jayllyz/labyrinth-game/issues/83)) - ([08de5b9](https://github.com/Jayllyz/labyrinth-game/commit/08de5b9feeefe7a995bddff216f2512778f80c96))
- *(radar)* Rename struct to fix duplicate ([#57](https://github.com/Jayllyz/labyrinth-game/issues/57)) - ([e3d9831](https://github.com/Jayllyz/labyrinth-game/commit/e3d983112611ee2b0bf8828b78a7b6e48f741574))
- *(server)* Improve based on client improvements ([#85](https://github.com/Jayllyz/labyrinth-game/issues/85)) - ([ca21b42](https://github.com/Jayllyz/labyrinth-game/commit/ca21b4214fae5a1c8101fde8896d73c199b83783))
- Remove unwrap usage ([#62](https://github.com/Jayllyz/labyrinth-game/issues/62)) - ([dec6e08](https://github.com/Jayllyz/labyrinth-game/commit/dec6e081a4ab3010983ba207b8ba6a41d69bd221))

### 🧪 Testing

- *(client)* Improve client code coverage - ([35068e2](https://github.com/Jayllyz/labyrinth-game/commit/35068e29ebe88e222730534375762030da52e8e7))
- Improve coverage and update e2e server ([#61](https://github.com/Jayllyz/labyrinth-game/issues/61)) - ([c9d94fa](https://github.com/Jayllyz/labyrinth-game/commit/c9d94fac0fe988b2789867bc1bb2a1f2f44f03ec))

### ⚙️ Miscellaneous Tasks

- *(workspace)* Update edition to 2024 - ([7f0e172](https://github.com/Jayllyz/labyrinth-game/commit/7f0e1723d5580895082b2cdb2cc071da0eb1395c))
- Update readme and document radar parser ([#93](https://github.com/Jayllyz/labyrinth-game/issues/93)) - ([3ee9420](https://github.com/Jayllyz/labyrinth-game/commit/3ee942001ff04c614b9946a1b367297a40846bac))
- Format - ([12b352d](https://github.com/Jayllyz/labyrinth-game/commit/12b352d242299fca6a236c49f6aac7fd9a6cfa5d))
- Configure nextest for ci runs ([#73](https://github.com/Jayllyz/labyrinth-game/issues/73)) - ([2b79c63](https://github.com/Jayllyz/labyrinth-game/commit/2b79c6309affbfd08c2644e7f72b24e75ad35ac0))
- Enable e2e on main and update changelog ([#69](https://github.com/Jayllyz/labyrinth-game/issues/69)) - ([1d10ad6](https://github.com/Jayllyz/labyrinth-game/commit/1d10ad6c1f99176b69bed69d6600249a549c78f6))
- Vscode debug setup - ([4c2dacd](https://github.com/Jayllyz/labyrinth-game/commit/4c2dacd4eb6bd8807d2765c249f2e58acae47314))

## [1.0.2](https://github.com/Jayllyz/labyrinth-game/compare/v1.0.1..v1.0.2) - 2024-12-17

### 📇 Features

- *(messages)* Add new messages ([#46](https://github.com/Jayllyz/labyrinth-game/issues/46)) - ([68df316](https://github.com/Jayllyz/labyrinth-game/commit/68df3160f3d5fddaceb3d9e3317bc2949e0be962))
- Solve secret sum challenge ([#49](https://github.com/Jayllyz/labyrinth-game/issues/49)) - ([a164e38](https://github.com/Jayllyz/labyrinth-game/commit/a164e38b4fbb44f0435ffb6370cbc43b2ed3f74e))

### 🐛 Bug Fixes

- *(messages)* Get action error to fix challenge edge case ([#54](https://github.com/Jayllyz/labyrinth-game/issues/54)) - ([eb5893c](https://github.com/Jayllyz/labyrinth-game/commit/eb5893c88fbafc234259e1b1cb7aa782a5470e0a))

### 🚜 Refactor

- *(client)* Improve readability of handle_server_message ([#55](https://github.com/Jayllyz/labyrinth-game/issues/55)) - ([728cc61](https://github.com/Jayllyz/labyrinth-game/commit/728cc61b1df6bf4329a756693096653f3842d84e))
- *(connection)* Rm mspc channel for token ([#48](https://github.com/Jayllyz/labyrinth-game/issues/48)) - ([2e011cf](https://github.com/Jayllyz/labyrinth-game/commit/2e011cff30250a907212252b35ea8e2a6344096b))

### ⚡ Performance

- *(radar)* Dont calculate len each times ([#52](https://github.com/Jayllyz/labyrinth-game/issues/52)) - ([166fcd4](https://github.com/Jayllyz/labyrinth-game/commit/166fcd4c96f855c77d804c0081f5409784e82afc))

### ⚙️ Miscellaneous Tasks

- *(e2e)* Update server ([#50](https://github.com/Jayllyz/labyrinth-game/issues/50)) - ([52567e7](https://github.com/Jayllyz/labyrinth-game/commit/52567e753dd99b8137d5894b90c5ed1bfd7a8c16))
- Fix warnings - ([c61b6c6](https://github.com/Jayllyz/labyrinth-game/commit/c61b6c6024cbbdd8bee3314a6ea4dd77a25f9cbe))
- Add zizmor check - ([2a82984](https://github.com/Jayllyz/labyrinth-game/commit/2a82984746062486b06eb64f89b33dc286fb42c6))
- Add git cliff changelog ([#47](https://github.com/Jayllyz/labyrinth-game/issues/47)) - ([ba1980b](https://github.com/Jayllyz/labyrinth-game/commit/ba1980be71ab89b339ff6b4aa21fab3b6092cbf8))

## [1.0.1](https://github.com/Jayllyz/labyrinth-game/compare/v1.0.0..v1.0.1) - 2024-12-03

### 🐛 Bug Fixes

- Archive bin name - ([a97eff7](https://github.com/Jayllyz/labyrinth-game/commit/a97eff7306fa85a2a900a27bddd376be28dc7c78))

## [1.0.0] - 2024-12-03

### 📇 Features

- *(archi)* Setup crates - ([796cbe8](https://github.com/Jayllyz/labyrinth-game/commit/796cbe8c7f99928d204f9b6aa64a26ab847cbd5f))
- *(client)* Run multiple agents in client ([#34](https://github.com/Jayllyz/labyrinth-game/issues/34)) - ([aea464e](https://github.com/Jayllyz/labyrinth-game/commit/aea464e12cbdd5e0f00722de5c6760080be12f89))
- *(client)* Basic right hand solving alg ([#27](https://github.com/Jayllyz/labyrinth-game/issues/27)) - ([bd631bd](https://github.com/Jayllyz/labyrinth-game/commit/bd631bd82de9b7979c536a3c82ffa7944a44a1b0))
- *(client)* Create maze parser ([#6](https://github.com/Jayllyz/labyrinth-game/issues/6)) - ([7edbbb7](https://github.com/Jayllyz/labyrinth-game/commit/7edbbb788f14ec470d4f28e77c429c5dddcf0467))
- *(maze)* Implement sidewinder generator ([#12](https://github.com/Jayllyz/labyrinth-game/issues/12)) - ([9308378](https://github.com/Jayllyz/labyrinth-game/commit/9308378ffc342188d3f7db71865f64b36ac88454))
- *(server)* Implement new registration logic ([#25](https://github.com/Jayllyz/labyrinth-game/issues/25)) - ([126fc25](https://github.com/Jayllyz/labyrinth-game/commit/126fc259dd46b1ec0d926cc3d1c5b5e4a370d045))
- *(server)* Setup teams and clients registration ([#8](https://github.com/Jayllyz/labyrinth-game/issues/8)) - ([bb9de0c](https://github.com/Jayllyz/labyrinth-game/commit/bb9de0c0c3de43c1660c7cfb24d9e46bc80fa3a7))
- *(shared)* Impl singleton logger ([#37](https://github.com/Jayllyz/labyrinth-game/issues/37)) - ([ea25861](https://github.com/Jayllyz/labyrinth-game/commit/ea258619451843b21e128d5cb70faf42621480cd))
- *(shared)* Create struct messages ([#3](https://github.com/Jayllyz/labyrinth-game/issues/3)) - ([e5004b7](https://github.com/Jayllyz/labyrinth-game/commit/e5004b77897df426620c855e5d0c6c7c11e0b00f))
- Check win conditions ([#28](https://github.com/Jayllyz/labyrinth-game/issues/28)) - ([57ec8b4](https://github.com/Jayllyz/labyrinth-game/commit/57ec8b42c110b6d182811cd8307335fbba922e34))
- Parse data from radar ([#26](https://github.com/Jayllyz/labyrinth-game/issues/26)) - ([d587de8](https://github.com/Jayllyz/labyrinth-game/commit/d587de80956d9348f8d7816ec28deecacd513f45))
- Handle team creation and registration token ([#23](https://github.com/Jayllyz/labyrinth-game/issues/23)) - ([3f043c3](https://github.com/Jayllyz/labyrinth-game/commit/3f043c34b82df8ab466279034b93fd7c0136d744))
- Print maze steps ([#16](https://github.com/Jayllyz/labyrinth-game/issues/16)) - ([bad3fbe](https://github.com/Jayllyz/labyrinth-game/commit/bad3fbeb636d2d04702d4d53b24b486b3fe3bbc8))
- Seed maze generation ([#15](https://github.com/Jayllyz/labyrinth-game/issues/15)) - ([9b7cc37](https://github.com/Jayllyz/labyrinth-game/commit/9b7cc3770579051e3c2bd33bd1c526e27643e8f3))
- Update args parameters and add offline mode ([#14](https://github.com/Jayllyz/labyrinth-game/issues/14)) - ([affe9dd](https://github.com/Jayllyz/labyrinth-game/commit/affe9dd091a9c09334a0f8b273092dbf9821ba74))
- Implement A* (a star) search algorithm returning the shortest path ([#13](https://github.com/Jayllyz/labyrinth-game/issues/13)) - ([70d5c21](https://github.com/Jayllyz/labyrinth-game/commit/70d5c21bb7a9ec672025ad931493c3f3229378b7))
- Setup bench suite ([#11](https://github.com/Jayllyz/labyrinth-game/issues/11)) - ([cc1ea2e](https://github.com/Jayllyz/labyrinth-game/commit/cc1ea2e00c764ab5caa8142559b69666392096ff))
- Setup clap cli ([#10](https://github.com/Jayllyz/labyrinth-game/issues/10)) - ([5334934](https://github.com/Jayllyz/labyrinth-game/commit/533493454c19c975f236228e2a19090d82ba757e))
- Implement bfs algorithm to return shortest path to exit a maze ([#9](https://github.com/Jayllyz/labyrinth-game/issues/9)) - ([d551029](https://github.com/Jayllyz/labyrinth-game/commit/d5510299decc1fd81411d48caa52218514fbe279))
- Handle client subscription ([#5](https://github.com/Jayllyz/labyrinth-game/issues/5)) - ([7bb6c4a](https://github.com/Jayllyz/labyrinth-game/commit/7bb6c4ab53f924c9e97c330732ae5cd6f5c7b241))
- Setup basic tcp server ([#4](https://github.com/Jayllyz/labyrinth-game/issues/4)) - ([0c517bf](https://github.com/Jayllyz/labyrinth-game/commit/0c517bf5a7573d914036835342612294eb981e7c))

### 🐛 Bug Fixes

- Workflow dispatch release - ([8259447](https://github.com/Jayllyz/labyrinth-game/commit/82594473520f893997ef1865ae73ff93cfa8ac90))
- Swapped vertical / horizontal ([#30](https://github.com/Jayllyz/labyrinth-game/issues/30)) - ([1d1973d](https://github.com/Jayllyz/labyrinth-game/commit/1d1973dfd72bd9e00f2d6fea8ea597937928abfb))
- Client & server communication ([#21](https://github.com/Jayllyz/labyrinth-game/issues/21)) - ([298fec6](https://github.com/Jayllyz/labyrinth-game/commit/298fec66dccc9272b010d8fed7ae5c5570f39fe5))

### 🚜 Refactor

- *(message)* Warn if cant read or receive ([#24](https://github.com/Jayllyz/labyrinth-game/issues/24)) - ([12b9cbc](https://github.com/Jayllyz/labyrinth-game/commit/12b9cbcadfeacdec231a1374d6b7e54d90cbe235))
- Use enum instead of a potential panic ([#17](https://github.com/Jayllyz/labyrinth-game/issues/17)) - ([75bc8f7](https://github.com/Jayllyz/labyrinth-game/commit/75bc8f71d7747ef46cd369c9f932fe2ca8670ff1))
- Improve request logging ([#7](https://github.com/Jayllyz/labyrinth-game/issues/7)) - ([8fc93bf](https://github.com/Jayllyz/labyrinth-game/commit/8fc93bfe9cbc25383b5a33d1e2dd2c277cfa4a15))

### ⚡ Performance

- *(radar)* Use binary operation ([#32](https://github.com/Jayllyz/labyrinth-game/issues/32)) - ([354a40a](https://github.com/Jayllyz/labyrinth-game/commit/354a40ae91f25397e58dbba4dce2ac3c3a00935e))

### 🎨 Styling

- Refactor string to enum ([#29](https://github.com/Jayllyz/labyrinth-game/issues/29)) - ([3db8c46](https://github.com/Jayllyz/labyrinth-game/commit/3db8c462d8f00379a4a8a61a2a5b6eddf05b57d8))

### 🧪 Testing

- Setup docker for e2e workflow - ([fbf4c06](https://github.com/Jayllyz/labyrinth-game/commit/fbf4c06265f62e90569f2fd80fb85d3148961bb8))

### ⚙️ Miscellaneous Tasks

- *(lint)* Add more clippy rules - ([e070f5a](https://github.com/Jayllyz/labyrinth-game/commit/e070f5ae8fee9f8c7e2b7a07f487e8277e2a9351))
- Setup release workflow ([#44](https://github.com/Jayllyz/labyrinth-game/issues/44)) - ([076745d](https://github.com/Jayllyz/labyrinth-game/commit/076745df1209547b6ca7ad3457e5ef7750415b2b))
- Run benchmark only on main ([#36](https://github.com/Jayllyz/labyrinth-game/issues/36)) - ([8db4b2b](https://github.com/Jayllyz/labyrinth-game/commit/8db4b2bcdf159ff5a8dfe1928ff66c220ecda73f))
- Rm print - ([f1b7bed](https://github.com/Jayllyz/labyrinth-game/commit/f1b7bed9cb1b47f4347687a10cd36c5e58982ab3))
- Create .vscode - ([b43d010](https://github.com/Jayllyz/labyrinth-game/commit/b43d0108e8ef9796ae91b8e7f812ab9e89997bf6))
- Add cargo lockfile - ([34e86fa](https://github.com/Jayllyz/labyrinth-game/commit/34e86fa678f8abba97e644024992e31ab38e96f5))
- Move renovate.json - ([7b319fe](https://github.com/Jayllyz/labyrinth-game/commit/7b319fe5885f04f7455508968d7364baa46080f7))
- Init repo - ([d596955](https://github.com/Jayllyz/labyrinth-game/commit/d596955d78a86bd399b2b16bdde3e2cb0c204db7))

