import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgForgotPasswordComponent } from './pg-forgot-password.component';

describe('PgForgotPasswordComponent', () => {
  let component: PgForgotPasswordComponent;
  let fixture: ComponentFixture<PgForgotPasswordComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PgForgotPasswordComponent]
    });
    fixture = TestBed.createComponent(PgForgotPasswordComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
