import { CdkTextareaAutosize } from "@angular/cdk/text-field";
import { CommonModule } from "@angular/common";
import {
    Component, ViewEncapsulation, ChangeDetectionStrategy, forwardRef, OnChanges, Input, ViewChild, SimpleChanges
} from "@angular/core";
import {
    ReactiveFormsModule, NG_VALUE_ACCESSOR, NG_VALIDATORS, FormControl, FormGroup, AbstractControl, ValidationErrors
} from "@angular/forms";
import { MatFormFieldModule, SubscriptSizing } from "@angular/material/form-field";
import { MatInputModule, MatInput } from "@angular/material/input";
import { TranslatePipe } from "@ngx-translate/core";
import { ValidatorUtils } from "../../utils/validator.utils";

export const TEXTAREA = "textarea";
export const CUSTOM_ERROR = "customError";

@Component({
    selector: "app-field-message",
    exportAs: "appFieldMessage",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, TranslatePipe, CdkTextareaAutosize],
    templateUrl: "./field-message.html",
    styleUrl: "./field-message.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldMessage), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldMessage), multi: true },
    ],
})
export class FieldMessage implements OnChanges {
    @Input()
    public errorMsg: string | null | undefined;
    @Input()
    public hint: string | null | undefined;
    @Input()
    public isDisabled: boolean | null | undefined;
    @Input()
    public isReadOnly: boolean | null | undefined;
    @Input()
    public isRequired: boolean | null | undefined;
    @Input()
    public isSpellcheck: boolean | null | undefined; // Whether to check for spelling and grammar errors.
    @Input()
    public kind: string | null | undefined = TEXTAREA;
    @Input()
    public label: string | null | undefined;
    @Input()
    public maxLen: number | null | undefined;
    @Input()
    public minLen: number | null | undefined;
    @Input()
    public maxRows: number | null | undefined;
    @Input()
    public minRows: number | null | undefined;
    @Input()
    public subscriptSizing: SubscriptSizing | null | undefined;

    @ViewChild(MatInput, { static: false })
    public matInput: MatInput | null = null;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ textarea: this.formControl });

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["isRequired"] || !!changes["minLen"] || !!changes["maxLen"]) {
            this.prepareFormGroup(this.isRequired || null, this.maxLen || null, this.minLen || null);
        }
        if (!!changes["isDisabled"]) {
            this.setDisabledState(!!this.isDisabled);
        }
        if (!!changes["errorMsg"]) {
            this.formControl.updateValueAndValidity();
            this.onChange(this.formControl.value);
        }
    }

    // ** ControlValueAccessor - start **

    public onChange: (val: string) => void = () => { };
    public onTouched: () => void = () => { };

    public writeValue(value: any): void {
        this.formControl.setValue(value, { emitEvent: true });
    }

    public registerOnChange(fn: any): void {
        this.onChange = fn;
    }

    public registerOnTouched(fn: any): void {
        this.onTouched = fn;
    }

    public setDisabledState(isDisabled: boolean): void {
        if (isDisabled != this.formGroup.disabled) {
            if (isDisabled) {
                this.formGroup.disable();
            } else {
                this.formGroup.enable();
            }
        }
    }

    // ** ControlValueAccessor - finish **

    // ** Validator - start **

    public validate(control: AbstractControl): ValidationErrors | null {
        return this.formControl.errors;
    }

    // ** Validator - finish **

    // ** Public API **

    public focus(): void {
        this.matInput?.focus();
    }

    public getErrorMsg(errors: ValidationErrors | null): string {
        const key = Object.keys(errors || {})[0];
        return !!key ? `417.${!!this.kind ? this.kind + ":" : ""}${key}` : "";
    }
    public getErrorObj(errors: ValidationErrors | null, value: string | null | undefined): ValidationErrors {
        const resObj = !!this.kind ? { [`${this.kind || ""}`]: value } : {};
        return { ...errors, ...resObj };
    }
    public getFormControl(): FormControl {
        return this.formControl;
    }
    public markAsTouched(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsTouched(opts);
    }
    public markAllAsTouched(opts?: { emitEvent?: boolean; }): void {
        this.formControl.markAllAsTouched(opts);
    }
    public markAsUntouched(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsUntouched(opts);
    }
    public markAsDirty(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsDirty(opts);
    }
    public markAsPristine(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsPristine(opts);
    }
    public markAsPending(opts?: { onlySelf?: boolean; emitEvent?: boolean; }): void {
        this.formControl.markAsPending(opts);
    }

    // ** Private API **

    private errorMsgValidator = (control: AbstractControl): ValidationErrors | null => {
        return !!control && !!this.errorMsg ? { [CUSTOM_ERROR]: true } : null;
    };
    private prepareFormGroup(isRequired: boolean | null, maxLen: number | null, minLen: number | null): void {
        this.formControl.clearValidators();
        const paramsObj = {
            ...(!!isRequired ? { "required": true } : {}),
            ...((minLen || 0) > 0 ? { "minLength": minLen } : {}),
            ...((maxLen || 0) > 0 ? { "maxLength": maxLen } : {}),
        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj), this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }
}
