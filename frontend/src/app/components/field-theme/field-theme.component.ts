import {
    ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewChild, ViewEncapsulation, forwardRef
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
    AbstractControl, ControlValueAccessor, FormControl, FormGroup, NG_VALIDATORS, NG_VALUE_ACCESSOR, ReactiveFormsModule,
    ValidationErrors, Validator, ValidatorFn, Validators
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelect, MatSelectModule } from '@angular/material/select';
import { TranslatePipe } from '@ngx-translate/core';

import { COLOR_SCHEME_LIST } from 'src/app/common/constants';

@Component({
    selector: 'app-field-theme',
    exportAs: 'appFieldTheme',
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatInputModule, MatFormFieldModule, MatSelectModule, TranslatePipe],
    templateUrl: './field-theme.component.html',
    styleUrl: './field-theme.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        { provide: NG_VALUE_ACCESSOR, useExisting: forwardRef(() => FieldThemeComponent), multi: true },
        { provide: NG_VALIDATORS, useExisting: forwardRef(() => FieldThemeComponent), multi: true },
    ],
})
export class FieldThemeComponent implements OnChanges, ControlValueAccessor, Validator {
    @Input()
    public isReadOnly: boolean = false;
    @Input()
    public isRequired: boolean = false;
    @Input()
    public label: string = 'field-theme.label';
    @Input()
    public hint: string = '';
    @Input()
    public isDisabled: boolean = false;

    @Output()
    readonly change: EventEmitter<string> = new EventEmitter();
    @Output()
    readonly openedChange: EventEmitter<boolean> = new EventEmitter();

    @ViewChild(MatSelect, { static: false })
    public matSelect: MatSelect | null = null;

    public formControl: FormControl = new FormControl({ value: null, disabled: false }, []);
    public formGroup: FormGroup = new FormGroup({ theme: this.formControl });
    public errMessage: string = '';

    public themeList: string[] = ['', ...COLOR_SCHEME_LIST];

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isRequired']) {
            this.prepareFormGroup();
        }
        if (!!changes['isDisabled']) {
            this.setDisabledState(this.isDisabled);
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
        isDisabled ? this.formGroup.disable() : this.formGroup.enable();
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

    public doOpenedChange(event: boolean): void {
        this.openedChange.emit(event);
    }

    public doSelectionChange(value: string): void {
        this.change.emit(value);
        this.onChange(this.formControl.value);
    }

    public getErrorMsg(errors: ValidationErrors | null): string {
        let result: string = '';
        const errorsProps: string[] = errors != null ? Object.keys(errors) : [];
        for (let index = 0; index < errorsProps.length && !result; index++) {
            const error: string = errorsProps[index];
            result = !result && 'required' === error ? 'ExpectationFailed.theme:required' : result;
        }
        return result;
    }

    // ** Private API **

    private prepareFormGroup(): void {
        this.formControl.clearValidators();
        const newValidator: ValidatorFn[] = [
            ...(this.isRequired ? [Validators.required] : []),
        ];
        this.formControl.setValidators(newValidator);
    }
}
