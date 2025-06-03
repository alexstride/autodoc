Each of these should be completed as a separate task/commit.

# Presentational changes

* The output should be printed in a scrollable terminal pane (like less or nano), not just printed to the terminal. There should be an option to quit by pressing q

# Functionality

* An autodoc.config file should indicate the root of the repo. Then every time the script runs it should search upwards until it finds the autodoc repo. The root of the repo should be where the file tree starts printing from, but all of the top level paths whicha are not part of the current query should appear in square brackets to illustrate that they are not actually relevant to the user's current query.
* There should be an ignore_paths config option, which shows paths which autodoc will never actually explore.

