import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, EventEmitter, forwardRef, Input, OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation
} from "@angular/core";
import {
    ReactiveFormsModule, NG_VALUE_ACCESSOR, NG_VALIDATORS, AbstractControl, ControlValueAccessor, FormControl, FormGroup, ValidationErrors, Validator,
} from "@angular/forms";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { MatSelectModule, MatSelect } from "@angular/material/select";
import { TranslatePipe } from "@ngx-translate/core";
import { LOCALE_LIST } from "../../common/locale-srv";
import { ValidatorUtils } from "../../utils/validator.utils";

export const LOCALE = "locale";
export const CUSTOM_ERROR = "customError";

@Component({
    selector: "app-field-locale",
    exportAs: "appFieldLocale",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatFormFieldModule,
        MatInputModule, MatSelectModule, TranslatePipe],
    templateUrl: "./field-locale.html",
    styleUrl: "./field-locale.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldLocale), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldLocale), multi: true },
    ],
})
export class FieldLocale implements OnChanges, ControlValueAccessor, Validator {
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
    public kind: string = LOCALE;
    @Input()
    public label: string | null | undefined;

    @Output()
    readonly change: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly openedChange: EventEmitter<boolean> = new EventEmitter();

    @ViewChild(MatSelect, { static: false })
    public matSelect: MatSelect | null = null;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ locale: this.formControl });

    public localeList: string[] = ["", ...LOCALE_LIST];

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["isRequired"]) {
            this.prepareFormGroup(this.isRequired || null);
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
        this.matSelect?.focus();
    }

    public getErrorMsg(errors: ValidationErrors | null): string {
        const key = Object.keys(errors || {})[0];
        return !!key ? `417.field-${this.kind || LOCALE}:${key}` : "";
    }
    public getErrorObj(errors: ValidationErrors | null, value: string | null | undefined): ValidationErrors {
        return { ...errors, ...{ [`field-${this.kind || LOCALE}`]: value } };
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

    public doOpenedChange(event: boolean): void {
        this.openedChange.emit(event);
    }

    public doSelectionChange(value: string): void {
        this.change.emit(value);
        this.onChange(this.formControl.value);
    }

    // ** Private API **

    private errorMsgValidator = (control: AbstractControl): ValidationErrors | null => {
        return !!control && !!this.errorMsg ? { [CUSTOM_ERROR]: true } : null;
    };

    private prepareFormGroup(isRequired: boolean | null): void {
        this.formControl.clearValidators();
        const paramsObj = {
            ...(!!isRequired ? { "required": true } : {}),
        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj), this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }
}
