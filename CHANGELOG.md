## [0.2.2](https://github.com/FAZuH/tomo/compare/v0.2.1...v0.2.2) (2026-05-01)


### fix

* **tui:** Duplicate input on Windows command prompt ([f34a18e](https://github.com/FAZuH/tomo/commit/f34a18e5dcd0db09122c0cdea528c5fc6cdcf629))


### perf

* **tui:** Improve initial draw speed ([99c9bc5](https://github.com/FAZuH/tomo/commit/99c9bc5e7e3858a36bf1c0150ecff9636a5adea0))

## [0.2.1](https://github.com/FAZuH/tomo/compare/v0.2.0...v0.2.1) (2026-05-01)


### perf

* **tui:** Fix tick timer bug causing high CPU usage ([d36489e](https://github.com/FAZuH/tomo/commit/d36489e37b93fd0b43ff85dd1143152cb3d5721a))
* **tui:** Improve idle CPU usage ([f74a09f](https://github.com/FAZuH/tomo/commit/f74a09f6a99d6896b060d000572e8e9b84454c46))
* **tui:** Redraw only when a valid input is pressed ([50c0e1d](https://github.com/FAZuH/tomo/commit/50c0e1ddc46f8e67129886b3298c6151ced8b3da))

## [0.2.0](https://github.com/FAZuH/tomo/compare/v0.1.7...v0.2.0) (2026-05-01)


### fix

* **tui:** Alarm volume settings showing alarm path when editing ([1a389a9](https://github.com/FAZuH/tomo/commit/1a389a914127bc3cb32a387ae67c4148428d1363))
* **tui:** Crash when toast exceeds frame height ([1de1a29](https://github.com/FAZuH/tomo/commit/1de1a2942819df43eb8e4cbc9d753786a333dc40))
* **tui:** Fix toast deduplication issues ([8c098d2](https://github.com/FAZuH/tomo/commit/8c098d2e53ca6ded0257bb8f42301e7f445da3df))


### feat

* **tui:** Add settings page keybind help ([8db8c50](https://github.com/FAZuH/tomo/commit/8db8c501ad92318257e2465e151a70a6902b585b))
* **tui:** Add settings section navigation and improve UI ([f628c27](https://github.com/FAZuH/tomo/commit/f628c271fd20d0cbd0adb65dc27fcfb280499a6a))
* **tui:** Add settings section select buttons ([3124353](https://github.com/FAZuH/tomo/commit/312435375e4b7db62bbef1c89a294c874f6005c2))
* **tui:** Add toast deduplication ([eaab8f0](https://github.com/FAZuH/tomo/commit/eaab8f0da4fe7371675d8b71d847a1bc594b823d))
* **tui:** Adjust padding of settings ([33fd694](https://github.com/FAZuH/tomo/commit/33fd69471d68587fd100028cd9c83cbbbac384e8))
* **tui:** Improve settings layout ([4024c54](https://github.com/FAZuH/tomo/commit/4024c544edb3b11bc822eb90e2fd42ff9ddf8ae4))
* **tui:** Improve timer page keybind hint ([8ffca11](https://github.com/FAZuH/tomo/commit/8ffca117a355b311921ca9aa06a856b12eb7b34c))
* **tui:** Invert timer 30sec offset keybinds ([d586abd](https://github.com/FAZuH/tomo/commit/d586abd445ebd8d8dbc318bee4872760e7da8750))
* **tui:** Make settings checkbox label dim ([08f8dcd](https://github.com/FAZuH/tomo/commit/08f8dcdf3b65a07c54b889f2e6bee24b33b1aa92))
* **tui:** Make timer shortcut toggleable ([8900478](https://github.com/FAZuH/tomo/commit/8900478530b941b3d986099d914814c3bd5784b4))
* **tui:** Trim percent when editing alarm volume ([83a47ec](https://github.com/FAZuH/tomo/commit/83a47ecee1437bec36c7bad6229306bd8a636bb0))

## [0.1.7](https://github.com/FAZuH/tomo/compare/v0.1.6...v0.1.7) (2026-04-28)


### feat

* Add warning/error toasts ([145dd17](https://github.com/FAZuH/tomo/commit/145dd172c91c512a104ee21a5a398b73a6b21fdb))
* **tui:** Add input validation to settings page ([679f389](https://github.com/FAZuH/tomo/commit/679f38962062174b135a05d812a3fde24449372d)), closes [#23](https://github.com/FAZuH/tomo/issues/23)
* **tui:** Add scroll input handling for settings page ([5a5f24a](https://github.com/FAZuH/tomo/commit/5a5f24a96f756166e28da81d184ad1af86126dac)), closes [#20](https://github.com/FAZuH/tomo/issues/20)
* **tui:** Improve settings input UI/UX ([05c1845](https://github.com/FAZuH/tomo/commit/05c18453f30e4731b4adfd40bd9571eb1d44dd25)), closes [#21](https://github.com/FAZuH/tomo/issues/21)


### perf

* Improve performance of data handling ([a9d19ce](https://github.com/FAZuH/tomo/commit/a9d19ce3f2cfa6dd9ccd96f65a08e70834201b93))

## [0.1.6](https://github.com/FAZuH/tomo/compare/v0.1.5...v0.1.6) (2026-04-27)


### refactor

* **core:** Set default long interval from 3 to 4 ([6bc3250](https://github.com/FAZuH/tomo/commit/6bc3250f2e400c551df90abb4eeb4eb9ea2b1abc))


### feat

* **core:** Add command hooks on session end ([71907e7](https://github.com/FAZuH/tomo/commit/71907e74ac78fba5079f3a8750f972bfb65d2b6d)), closes [#4](https://github.com/FAZuH/tomo/issues/4)
* **core:** Notify on session transitions ([45b8e90](https://github.com/FAZuH/tomo/commit/45b8e90deea6c34306fcea34ae653614dd25d500))

