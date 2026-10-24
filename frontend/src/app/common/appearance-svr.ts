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

const FORMFIELD_APPEARANCE_DEFAULT = "outline";
const BUTTON_APPEARANCE_DEFAULT = "tonal";

export class SettingsUtil {
    public static create(data: Partial<SettingsDto> | null | undefined): SettingsDto {
        const dataObj = data || {};
        const result: SettingsDto = {
            formFieldAppearance: this.checkFormFieldAppearance(dataObj.formFieldAppearance),
            formFieldShape: this.checkFormFieldShape(dataObj.formFieldShape),
            buttonAppearance: this.checkButtonAppearance(dataObj.buttonAppearance),
            buttonBorder: this.checkButtonBorder(dataObj.buttonBorder),
            buttonShape: this.checkButtonShape(dataObj.buttonShape),
        };
        return result;
    }
    public static checkFormFieldAppearance(value: string | null | undefined): MatFormFieldAppearance {
        const result: MatFormFieldAppearance | undefined =
            value == "fill" ? "fill" :
                value == "outline" ? "outline" : undefined;
        return result || FORMFIELD_APPEARANCE_DEFAULT;
    }

    public static checkFormFieldShape(value: string | null | undefined): string | undefined {
        const val = value?.toLowerCase();
        return !!val && FORM_FIELD_SHAPE_LIST.includes(val) ? val : undefined;
    }
    public static checkButtonAppearance(value: string | null | undefined): MatButtonAppearance {
        const result: MatButtonAppearance | undefined =
            value == "text" ? "text" :
                value == "filled" ? "filled" :
                    value == "elevated" ? "elevated" :
                        value == "outlined" ? "outlined" :
                            value == "tonal" ? "tonal" : undefined;
        return result || BUTTON_APPEARANCE_DEFAULT;
    }
    public static checkButtonBorder(value: boolean | null | undefined): boolean {
        return value ?? true;
    }
    public static checkButtonShape(value: string | null | undefined): string | undefined {
        const val = value?.toLowerCase();
        return !!val && BUTTON_SHAPE_LIST.indexOf(val) > -1 ? val : undefined;
    }
}

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

    public getFromStorage(): SettingsDto | null {
        const settingsStr = localStorage.getItem(PATH_SETTINGS);
        const data = !!settingsStr ? JSON.parse(settingsStr) : null;
        const settings = !data ? data : {
            formFieldAppearance: data?.formFieldAppearance,
            formFieldShape: data?.formFieldShape,
            buttonAppearance: data?.buttonAppearance,
            buttonBorder: data?.buttonBorder,
            buttonShape: data?.buttonShape,
        };
        return settings;
    }

    public saveToStorage(settings: SettingsDto | null | undefined = this.getSettings()): void {
        const settingsStr = !!settings ? JSON.stringify(settings) : "";
        if (!!settingsStr && settingsStr != "{}") {
            window.localStorage.setItem(PATH_SETTINGS, settingsStr);
        } else {
            window.localStorage.removeItem(PATH_SETTINGS);
        }
    }

    // ** FormFieldAppearance **

    public setFormFieldAppearance(value: string | null | undefined): void {
        const val = SettingsUtil.checkFormFieldAppearance(value);
        if (val != this.appFormFieldDefaultOptions.appearance) {
            this.appFormFieldDefaultOptions.appearance = val;
            this.settings = { ...this.settings, ...{ formFieldAppearance: val } };
            Object.freeze(this.settings);
        }
    }

    public getFormFieldAppearance(): string {
        return this.settings.formFieldAppearance || "";
    }

    // ** FormFieldShape **

    public setFormFieldShape(value: string | null | undefined): void {
        const val = SettingsUtil.checkFormFieldShape(value);
        if (val != this.settings.formFieldShape) {
            this.settings = { ...this.settings, ...{ formFieldShape: val } };
            Object.freeze(this.settings);
            const element: HTMLElement = this.document.body;
            element.style.setProperty(MAT_FORM_FIELD_OUTLINED_CONTAINER_SHAPE, val || null);
            element.style.setProperty(MAT_FORM_FIELD_FILLED_CONTAINER_SHAPE, val || null);
        }
    }
    public getFormFieldShape(): string {
        return this.settings.formFieldShape || "";
    }

    // ** ButtonAppearance **

    public setButtonAppearance(value: string | null | undefined): void {
        const val = SettingsUtil.checkButtonAppearance(value);
        if (val != this.settings.buttonAppearance) {
            this.appButtonConfig.defaultAppearance = val;
            this.settings = { ...this.settings, ...{ buttonAppearance: val } };
            Object.freeze(this.settings);
        }
    }

    public getButtonAppearance(): string {
        return this.settings.buttonAppearance || "";
    }

    // ** ButtonBorder **

    public setButtonBorder(isAdd: boolean | null | undefined): void {
        const isAdd2 = SettingsUtil.checkButtonBorder(isAdd);
        if (isAdd2 !== !!this.settings.buttonBorder) {
            const element: HTMLElement = this.document.body;
            if (!!isAdd2) {
                element.classList.add("stl-btn");
            } else {
                element.classList.remove("stl-btn");
            }
            this.settings = { ...this.settings, ...{ buttonBorder: !!isAdd2 } };
            Object.freeze(this.settings);
        }
    }
    public getButtonBorder(): boolean {
        return !!this.settings.buttonBorder;
    }

    // ** ButtonShape **

    public setButtonShape(value: string | null | undefined): void {
        const val = SettingsUtil.checkButtonShape(value);
        if (val != this.settings.buttonShape) {
            const element: HTMLElement = this.document.body;
            element.style.setProperty(MAT_BUTTON_TEXT_CONTAINER_SHAPE, val || null);
            element.style.setProperty(MAT_BUTTON_PROTECTED_CONTAINER_SHAPE, val || null);
            element.style.setProperty(MAT_BUTTON_OUTLINED_CONTAINER_SHAPE, val || null);
            element.style.setProperty(MAT_BUTTON_FILLED_CONTAINER_SHAPE, val || null);
            element.style.setProperty(MAT_BUTTON_TONAL_CONTAINER_SHAPE, val || null);
            this.settings = { ...this.settings, ...{ buttonShape: val } };
            Object.freeze(this.settings);
        }
    }
    public getButtonShape(): string {
        return this.settings.buttonShape || "";
    }

    // ** Private API **
}
