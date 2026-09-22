import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges, TemplateRef, ViewChild,
    ViewContainerRef, ViewEncapsulation
} from "@angular/core";
import { FormControl, FormGroup, ReactiveFormsModule } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatCheckboxModule } from "@angular/material/checkbox";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatInputModule } from "@angular/material/input";
import { MatSelectModule } from "@angular/material/select";
import { TranslatePipe } from "@ngx-translate/core";
import { FORM_FIELD_APPEARANCE_LIST, AppearanceSvr, BUTTON_SHAPE_LIST, FORM_FIELD_SHAPE_LIST, BUTTON_APPEARANCE_LIST } from "../../common/appearance-svr";
import { SettingsDto } from "../../lib-user/user-dto";
import { ErrMsgObj } from "../../utils/http-error.util";

@Component({
    selector: "app-panel-profile-settings",
    exportAs: "appPanelProfileSettings",
    standalone: true,
    imports: [CommonModule, ReactiveFormsModule, MatButtonModule, MatCheckboxModule, MatInputModule, TranslatePipe, MatFormFieldModule,
        MatSelectModule],
    templateUrl: "./panel-profile-settings.html",
    styleUrl: "./panel-profile-settings.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelProfileSettings implements OnChanges {
    private appearanceSvr: AppearanceSvr = inject(AppearanceSvr);

    @Input()
    public errMsgObjs: ErrMsgObj[] = [];
    @Input()
    public isDisabled: boolean | null | undefined;
    @Input()
    public settingsDto: SettingsDto | null | undefined;

    @Output()
    readonly changeData: EventEmitter<boolean> = new EventEmitter();
    @Output()
    readonly updateSettings: EventEmitter<SettingsDto> = new EventEmitter();

    @ViewChild("outlet", { read: ViewContainerRef })
    public outletRef!: ViewContainerRef;
    @ViewChild("content", { read: TemplateRef })
    public contentRef!: TemplateRef<unknown>;

    public cntls = {
        formFieldAppearance: new FormControl("", []),
        formFieldShape: new FormControl("", []),
        buttonAppearance: new FormControl("", []),
        buttonBorder: new FormControl(false, []),
        buttonShape: new FormControl("", []),
    };
    public formGroup: FormGroup = new FormGroup(this.cntls);

    public formFieldAppearanceList = [...FORM_FIELD_APPEARANCE_LIST];
    public formFieldShapeList = ["", ...FORM_FIELD_SHAPE_LIST];
    public buttonAppearanceList = ["", ...BUTTON_APPEARANCE_LIST];
    public buttonShapeList = ["", ...BUTTON_SHAPE_LIST];

    private isChangeData: boolean = false;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["settingsDto"]) {
            const settings = {
                formFieldAppearance: (this.settingsDto?.formFieldAppearance || this.appearanceSvr.getFormFieldAppearance()),
                formFieldShape: (this.settingsDto?.formFieldShape || ""),
                buttonAppearance: (this.settingsDto?.buttonAppearance || this.appearanceSvr.getButtonAppearance()),
                buttonBorder: (this.settingsDto?.buttonBorder || false),
                buttonShape: (this.settingsDto?.buttonShape || ""),
            };
            this.formGroup.patchValue(settings);
            this.formGroup.markAsPristine();
        }
    }

    // ** Public API **

    public doChangeData(name: string, formGroup: FormGroup, origValues: unknown): void {
        const origValues2 = (origValues as Record<string, string>) || {};
        if (!name || !formGroup) {
            return;
        }
        if ((formGroup.controls[name]?.value || "") == (origValues2[name] || "")) {
            formGroup.controls[name].markAsPristine();
        }
        if (this.isChangeData != formGroup.dirty) {
            this.changeData.emit(this.isChangeData = formGroup.dirty);
        }
    }

    // ** FormField **

    public setFormFieldAppearance(value: string): void {
        this.appearanceSvr.setFormFieldAppearance(value);
        this.rerendering();
    }

    public setFormFieldShape(value: string): void {
        this.appearanceSvr.setFormFieldShape(value);
    }

    // ** Button **

    public setButtonAppearance(value: string): void {
        this.appearanceSvr.setButtonAppearance(value);
        this.rerendering();
    }

    public setButtonBorder(isAdd: boolean): void {
        this.appearanceSvr.setButtonBorder(isAdd);
    }

    public setButtonShape(value: string): void {
        this.appearanceSvr.setButtonShape(value);
    }

    // ** -- **

    public updateErrMsgObjs(errMsgObjs: ErrMsgObj[] = []): void {
        this.errMsgObjs = errMsgObjs;
    }

    public saveSettings(formGroup: FormGroup, isDisabled: boolean): void {
        if (!formGroup || formGroup.pristine || formGroup.invalid || isDisabled) {
            return;
        }
        const modifySettings: SettingsDto = {
            formFieldAppearance: formGroup.get("formFieldAppearance")?.value,
            formFieldShape: formGroup.get("formFieldShape")?.value,
            buttonAppearance: formGroup.get("buttonAppearance")?.value,
            buttonBorder: formGroup.get("buttonBorder")?.value,
            buttonShape: formGroup.get("buttonShape")?.value,
        };
        const is_all_empty = Object.values(modifySettings).findIndex((value) => value !== undefined) == -1;
        if (!is_all_empty) {
            this.updateSettings.emit(modifySettings);
        }
    }

    // ** Private API **

    private rerendering(): void {
        this.outletRef?.clear();
        this.outletRef?.createEmbeddedView(this.contentRef);
    }
}
