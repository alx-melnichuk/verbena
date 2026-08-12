import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelForgotPassword } from './panel-forgot-password';

describe('PanelForgotPassword', () => {
  let component: PanelForgotPassword;
  let fixture: ComponentFixture<PanelForgotPassword>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelForgotPassword]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelForgotPassword);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
