import { text } from "@sveltejs/kit";




function createConfirmationScreenStore() {

    let isOpen = $state(false);
    let dialogTitle = $state("");
    let dialogText = $state("");
    let confirmButtonText = $state("Confirm");
    let exitButtonText = $state("exit");
    let passUnsafeHTML = $state(false)
    let confirmCallback: Function | undefined = $state();



    return {
        get isOpen() { return isOpen },
        get title() { return dialogTitle },
        get text() { return dialogText },
        get confirmText() { return confirmButtonText },
        get exitText() { return exitButtonText },
        get passUnsafeHTML() { return passUnsafeHTML },



        /**
         * 
         * @param title 
         * @param text 
         * @param _confirmCallback 
         * @param _confirmButtonText 
         * @param _exitButtonText 
         * @param _passUnsafeHTML Do not use this one with user input   
         */
        newDialog: (title: string, text: string, _confirmCallback: Function, _confirmButtonText?: string, _exitButtonText?: string, _passUnsafeHTML = false) => {
            passUnsafeHTML = _passUnsafeHTML
            // open the dialog after setting unsafe html, just in case 
            isOpen = true;
            dialogTitle = title;
            dialogText = text;
            confirmCallback = _confirmCallback;


            if (typeof _confirmButtonText !== "undefined") {
                confirmButtonText = _confirmButtonText;
            } else {
                confirmButtonText = "Confirm"
            }

            if (typeof _exitButtonText !== "undefined") {
                exitButtonText = _exitButtonText;
            } else {
                exitButtonText = "Exit"
            }
        },

        onConfirm: () => {

            if (typeof confirmCallback !== undefined) {
                confirmCallback!!() // lol no optionals
                ConfirmationScreenStore.close()

            } else {
                console.error("Tried to call the dialog callback without a callback being set, are you sure you have created the dialog?")
            }
        },

        close: () => {
            passUnsafeHTML = false;
            isOpen = false;
        }

        /*
        open: (hash: string) => {
            if (!isOpen) {
                selectedHash = hash;
                isOpen = true;
            }
    
        },
        close: () => {
            isOpen = false;
            tagsEditModeActive = false;
        },
    
        setTagsEditModeActive(state: boolean) {
            tagsEditModeActive = state
        }
            */
    }

}

export const ConfirmationScreenStore = createConfirmationScreenStore();
