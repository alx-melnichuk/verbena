import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelSignupComponent } from './panel-signup.component';

describe('PanelSignupComponent', () => {
  let component: PanelSignupComponent;
  let fixture: ComponentFixture<PanelSignupComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelSignupComponent]
    });
    fixture = TestBed.createComponent(PanelSignupComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
