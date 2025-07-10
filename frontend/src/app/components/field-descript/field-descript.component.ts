import {
    ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewChild, ViewEncapsulation, forwardRef,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
    AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule,
    ValidationErrors, Validator, ValidatorFn,
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInput, MatInputModule } from '@angular/material/input';
import { TranslatePipe } from '@ngx-translate/core';

import { ValidatorUtils } from 'src/app/utils/validator.utils';

export const DESCRIPT = 'descript';
export const DESCRIPT_MIN_LENGTH = 2;
export const DESCRIPT_MAX_LENGTH = 2048; // 2*1024
export const DESCRIPT_ROWS = 6;
export const CUSTOM_ERROR = 'customError';

@Component({
    selector: 'app-field-descript',
    exportAs: 'appFieldDescript',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, TranslatePipe],
    templateUrl: './field-descript.component.html',
    styleUrl: './field-descript.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldDescriptComponent), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldDescriptComponent), multi: true },
    ],
})
export class FieldDescriptComponent implements OnChanges, ControlValueAccessor, Validator {
    @Input()
    public gist: string = DESCRIPT;
    @Input()
    public errorMsg: string | null | undefined;
    @Input()
    public hint: string = '';
    @Input()
    public isDisabled: boolean = false;
    @Input()
    public isReadOnly: boolean = false;
    @Input()
    public isRequired: boolean = false;
    @Input()
    public isSpellcheck: boolean = false;
    @Input()
    public label: string = 'field-descript.label';
    @Input()
    public maxLen: number = DESCRIPT_MAX_LENGTH;
    @Input()
    public minLen: number = DESCRIPT_MIN_LENGTH;
    @Input()
    public numberRows = DESCRIPT_ROWS;

    @ViewChild(MatInput, { static: false })
    public matInput: MatInput | null = null;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ description: this.formControl });

    constructor() {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isRequired'] || !!changes['minLen'] || !!changes['maxLen']) {
            this.prepareFormGroup(this.isRequired, this.maxLen, this.minLen);
        }
        if (!!changes['isDisabled']) {
            this.setDisabledState(this.isDisabled);
        }
        if (!!changes['errorMsg']) {
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
        return ValidatorUtils.getErrorMsg(errors, this.gist || DESCRIPT);
    }

    // ** Private API **

    private errorMsgValidator = (control: AbstractControl): ValidationErrors | null => {
        const result = !!control && !!this.errorMsg ? { [CUSTOM_ERROR]: true } : null;
        return result;
    };
    private prepareFormGroup(isRequired: boolean, maxLen: number, minLen: number): void {
        this.formControl.clearValidators();
        const paramsObj = {
            ...(isRequired ? { "required": true } : {}),
            ...(minLen > 0 ? { "minLength": minLen } : {}),
            ...(maxLen > 0 ? { "maxLength": maxLen } : {}),
        };
        this.formControl.setValidators([...ValidatorUtils.prepare(paramsObj), this.errorMsgValidator]);
        this.formControl.updateValueAndValidity();
    }
}
