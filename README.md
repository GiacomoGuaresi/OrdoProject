# OrdoProject

**OrdoProject** is a lightweight utility application that helps you keep
your files organized automatically.\
With a simple and intuitive interface, you can define rules based on
**wildcards** and map them to **target directories**.\
When applied to a file, folder, or even your entire desktop, OrdoProject
automatically sorts and moves files according to your rules.

------------------------------------------------------------------------

## ✨ Features

-   🗂 **Rule-based sorting** -- Define wildcard rules (e.g. `*.jpg`,
    `report_*.pdf`) and associate them with target directories.\
-   📂 **Batch organization** -- Apply rules to single files, selected
    folders, or your desktop.\
-   🔄 **Automatic moving** -- Files matching the rules are instantly
    moved into their corresponding directories.\
-   📜 **Logging** -- Keep track of all file movements with a detailed
    log.\
-   🎨 **Modern UI** -- Built with **Electron**, **React**, and
    **Material Design** for a clean and user-friendly experience.

------------------------------------------------------------------------

## 🚀 Installation

1.  Clone the repository:

    ``` bash
    git clone https://github.com/yourusername/OrdoProject.git
    ```

2.  Navigate to the project folder:

    ``` bash
    cd OrdoProject
    ```

3.  Install dependencies:

    ``` bash
    npm install
    ```

4.  Start the application in development mode:

    ``` bash
    npm start
    ```

To build the production version:

``` bash
npm run build
```

------------------------------------------------------------------------

## 📖 Usage

1.  Open **OrdoProject**.\
2.  Create one or more rules by specifying:
    -   **Pattern (wildcard)** → e.g. `*.png`, `music_*.mp3`\
    -   **Target directory** → where the matched files should go\
3.  Select a file, a folder, or your desktop as the source.\
4.  Apply rules -- OrdoProject will automatically move the files.\
5.  Check the **log** to review the performed operations.

------------------------------------------------------------------------

## 🛠 Tech Stack

-   **Electron** -- Desktop app framework\
-   **React** -- Frontend library\
-   **Material UI** -- UI components\
-   **Node.js** -- Backend logic

------------------------------------------------------------------------

## 📌 Roadmap / Ideas

-   Add **regex-based rules** in addition to wildcards.\
-   Enable **scheduled automatic sorting** (background service).\
-   Provide **cloud sync** for rules across devices.\
-   Allow **undo** operations for recent file movements.

------------------------------------------------------------------------

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!\
Feel free to open a pull request or file an issue in the repository.

------------------------------------------------------------------------

## 📄 License

This project is licensed under the [MIT License](LICENSE).
