dev:
	cargo run

test:
	cargo test

git:
	@git remote add dokku dokku@ssh.kbl.io:rhxn

deploy:
	@git push dokku main
