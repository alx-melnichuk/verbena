import { DOCUMENT, inject, Injectable } from "@angular/core";
import { MatFormFieldDefaultOptions, MAT_FORM_FIELD_DEFAULT_OPTIONS, MatFormFieldAppearance } from "@angular/material/form-field";
import { environment as env } from "../../environments/environment";
import { Settings } from "./session-srv";

export const PATH_SETTINGS = "settings";

export const APPEARANCE_FILL = "fill";
export const APPEARANCE_OUTLINE = "outline";
export const APPEARANCE_LIST = [APPEARANCE_OUTLINE, APPEARANCE_FILL];

export const FORM_FIELD_SHAPE_LIST = ["0px", "0.25rem", "0.5rem", "0.75rem", "1rem", "1.5rem", "2rem"];
const MAT_FORM_FIELD_OUTLINED_CONTAINER_SHAPE = "--mat-form-field-outlined-container-shape";
const MAT_FORM_FIELD_FILLED_CONTAINER_SHAPE = "--mat-form-field-filled-container-shape";

export const BUTTON_SHAPE_LIST = [
    "0px",
    "0.4rem / 1.0rem", "0.6rem / 1.5rem", "0.8rem / 1.5rem", "1.0rem / 1.5rem",
    "1.4rem / 1.4rem", "1.1rem / 1.1rem", "0.8rem / 0.8rem", "0.5rem / 0.5rem",
];
const MAT_BUTTON_TEXT_CONTAINER_SHAPE = "--mat-button-text-container-shape";
const MAT_BUTTON_PROTECTED_CONTAINER_SHAPE = "--mat-button-protected-container-shape";
const MAT_BUTTON_OUTLINED_CONTAINER_SHAPE = "--mat-button-outlined-container-shape";
const MAT_BUTTON_FILLED_CONTAINER_SHAPE = "--mat-button-filled-container-shape";
const MAT_BUTTON_TONAL_CONTAINER_SHAPE = "--mat-button-tonal-container-shape";

@Injectable({
    providedIn: "root",
})
export class AppearanceSvr {
    private document: Document = inject(DOCUMENT);
    private appFormFieldDefaultOptions: MatFormFieldDefaultOptions = inject(MAT_FORM_FIELD_DEFAULT_OPTIONS);
    private settings: Settings = {};

    constructor() {
        if (env.logLevel & 4) { console.info(`AppearanceSvr(); // 6 service`); }
    }

    // ** Public API **

    public getSettings(): Settings {
        return { ...this.settings };
    }
    public setSettings(value: Settings | null | undefined): void {
        this.settings = value || {};
        this.setAppearance(this.settings.appearance);
        this.setFormFieldShape(this.settings.formFieldShape);
        this.setButtonShape(this.settings.buttonShape);
    }

    // ** Appearance **

    public checkAppearance(value: string | null | undefined): MatFormFieldAppearance | undefined {
        return ((value || "").toLowerCase() == "fill" ? "fill" : "outline");
    }
    public setAppearance(value: string | null | undefined): void {
        const val = this.checkAppearance(value);
        this.appFormFieldDefaultOptions.appearance = val;
        this.settings.appearance = val;
    }

    public getAppearance(): string {
        return this.settings.appearance || "";
    }

    // ** FormFieldShape **

    public setFormFieldShape(value: string | null | undefined): void {
        const val = value?.toLowerCase();
        const val1 = (!!val && FORM_FIELD_SHAPE_LIST.includes(val) ? val : null);
        this.settings.formFieldShape = val1 || undefined;
        const element: HTMLElement = this.document.body;
        element.style.setProperty(MAT_FORM_FIELD_OUTLINED_CONTAINER_SHAPE, val1);
        element.style.setProperty(MAT_FORM_FIELD_FILLED_CONTAINER_SHAPE, val1);
    }
    public getFormFieldShape(): string {
        return this.settings.formFieldShape || "";
    }

    // ** ButtonShape **

    public setButtonShape(value: string | null | undefined): void {
        const val = value?.toLowerCase();
        const val1 = (!!val && BUTTON_SHAPE_LIST.indexOf(val) > -1 ? val : null);
        this.settings.buttonShape = val1 || undefined;
        const element: HTMLElement = this.document.body;
        element.style.setProperty(MAT_BUTTON_TEXT_CONTAINER_SHAPE, val1);
        element.style.setProperty(MAT_BUTTON_PROTECTED_CONTAINER_SHAPE, val1);
        element.style.setProperty(MAT_BUTTON_OUTLINED_CONTAINER_SHAPE, val1);
        element.style.setProperty(MAT_BUTTON_FILLED_CONTAINER_SHAPE, val1);
        element.style.setProperty(MAT_BUTTON_TONAL_CONTAINER_SHAPE, val1);
    }
    public getButtonShape(): string {
        return this.settings.buttonShape || "";
    }

    // ** Private API **
}
