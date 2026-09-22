import { DOCUMENT, inject, Injectable } from "@angular/core";
import { MAT_BUTTON_CONFIG, MatButtonAppearance, MatButtonConfig } from "@angular/material/button";
import { MatFormFieldDefaultOptions, MAT_FORM_FIELD_DEFAULT_OPTIONS, MatFormFieldAppearance } from "@angular/material/form-field";
import { environment as env } from "../../environments/environment";
import { SettingsDto } from "../lib-user/user-dto";

export const PATH_SETTINGS = "settings";

export const FORM_FIELD_APPEARANCE_LIST = ["outline", "fill"];

export const FORM_FIELD_SHAPE_LIST = ["0px", "0.25rem", "0.5rem", "0.75rem", "1rem", "1.5rem", "2rem"];
const MAT_FORM_FIELD_OUTLINED_CONTAINER_SHAPE = "--mat-form-field-outlined-container-shape";
const MAT_FORM_FIELD_FILLED_CONTAINER_SHAPE = "--mat-form-field-filled-container-shape";

export const BUTTON_APPEARANCE_LIST = ["text", "filled", "elevated", "outlined", "tonal"];

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
    private appButtonConfig: MatButtonConfig = inject(MAT_BUTTON_CONFIG);
    private settings: SettingsDto = {};

    constructor() {
        if (env.logLevel & 4) { console.info(`AppearanceSvr(); // 6 service`); }
    }

    // ** Public API **

    public getSettings(): SettingsDto {
        return this.settings;
    }
    public setSettings(value: SettingsDto | null | undefined): void {
        const settings = value || {};
        this.setFormFieldAppearance(settings.formFieldAppearance);
        this.setFormFieldShape(settings.formFieldShape);
        this.setButtonAppearance(settings.buttonAppearance);
        this.setButtonBorder(settings.buttonBorder);
        this.setButtonShape(settings.buttonShape);
    }

    // ** FormFieldAppearance **

    public checkFormFieldAppearance(value: string | null | undefined): MatFormFieldAppearance | undefined {
        const val = (value || "").toLowerCase();
        const result: MatFormFieldAppearance | undefined =
            val == "fill" ? "fill" :
                val == "outline" ? "outline" : undefined;
        return result;
    }
    public setFormFieldAppearance(value: string | null | undefined): void {
        const val = this.checkFormFieldAppearance(value) || "outline";
        this.appFormFieldDefaultOptions.appearance = val;
        this.settings = { ...this.settings, ...{ formFieldAppearance: val } };
        Object.freeze(this.settings);
    }

    public getFormFieldAppearance(): string {
        return this.settings.formFieldAppearance || "";
    }

    // ** FormFieldShape **

    public setFormFieldShape(value: string | null | undefined): void {
        const val = value?.toLowerCase();
        const val1 = (!!val && FORM_FIELD_SHAPE_LIST.includes(val) ? val : null);
        this.settings = { ...this.settings, ...{ formFieldShape: (val1 || undefined) } };
        Object.freeze(this.settings);
        const element: HTMLElement = this.document.body;
        element.style.setProperty(MAT_FORM_FIELD_OUTLINED_CONTAINER_SHAPE, val1);
        element.style.setProperty(MAT_FORM_FIELD_FILLED_CONTAINER_SHAPE, val1);
    }
    public getFormFieldShape(): string {
        return this.settings.formFieldShape || "";
    }

    // ** ButtonAppearance **

    public checkButtonAppearance(value: string | null | undefined): MatButtonAppearance | undefined {
        const val = (value || "").toLowerCase();
        const result: MatButtonAppearance | undefined =
            val == "text" ? "text" :
                val == "filled" ? "filled" :
                    val == "elevated" ? "elevated" :
                        val == "outlined" ? "outlined" :
                            val == "tonal" ? "tonal" : undefined;
        return result;
    }
    public setButtonAppearance(value: string | null | undefined): void {
        const val = this.checkButtonAppearance(value) || "outlined";
        this.appButtonConfig.defaultAppearance = val;
        this.settings = { ...this.settings, ...{ buttonAppearance: val } };
        Object.freeze(this.settings);
    }

    public getButtonAppearance(): string {
        return this.settings.buttonAppearance || "";
    }

    // ** ButtonShape **

    public setButtonShape(value: string | null | undefined): void {
        const val = value?.toLowerCase();
        const val1 = (!!val && BUTTON_SHAPE_LIST.indexOf(val) > -1 ? val : null);
        this.settings = { ...this.settings, ...{ buttonShape: (val1 || undefined) } };
        Object.freeze(this.settings);
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

    // ** ButtonBorder **

    public setButtonBorder(isAdd: boolean | null | undefined): void {
        this.settings = { ...this.settings, ...{ buttonBorder: !!isAdd } };
        Object.freeze(this.settings);
        const element: HTMLElement = this.document.body;
        if (!!isAdd) {
            element.classList.add("stl-btn");
        } else {
            element.classList.remove("stl-btn");
        }
    }
    public getButtonBorder(): boolean {
        return !!this.settings.buttonBorder;
    }

    // ** Private API **
}
