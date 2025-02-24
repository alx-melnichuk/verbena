import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelForgotPasswordComponent } from './panel-forgot-password.component';

describe('PanelForgotPasswordComponent', () => {
  let component: PanelForgotPasswordComponent;
  let fixture: ComponentFixture<PanelForgotPasswordComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelForgotPasswordComponent]
    });
    fixture = TestBed.createComponent(PanelForgotPasswordComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
