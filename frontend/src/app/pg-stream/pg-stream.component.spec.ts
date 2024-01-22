import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgStreamComponent } from './pg-stream.component';

describe('PgStreamComponent', () => {
  let component: PgStreamComponent;
  let fixture: ComponentFixture<PgStreamComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PgStreamComponent]
    });
    fixture = TestBed.createComponent(PgStreamComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
