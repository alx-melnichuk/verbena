import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgAboutComponent } from './pg-about.component';

describe('PgAboutComponent', () => {
  let component: PgAboutComponent;
  let fixture: ComponentFixture<PgAboutComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PgAboutComponent]
    });
    fixture = TestBed.createComponent(PgAboutComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
